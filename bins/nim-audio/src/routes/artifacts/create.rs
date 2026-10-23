use heck::ToSnakeCase;
use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    let bytes = match &signal.content.content {
        nim::state::ArtifactContent::Text { text } => text.as_bytes().to_vec(),
        nim::state::ArtifactContent::File {
            path,
            size: _,
            media_type: _,
        } => tokio::fs::read(path).await?.to_vec(),
    };

    let captions = nim::ai::pipelines::audio::captions::run(std::slice::from_ref(&bytes))?;
    let Some(caption) = captions.first() else {
        return Err(Error::custom(format!(
            "no captions could be generated from artifact `{}`",
            signal.content.id
        )));
    };

    let memory = if let Some(embedding) = caption.vector.as_deref() {
        ctx.services()
            .memories()
            .search(embedding, nim::state::MemoryType::Working, 10)
            .await?
            .first()
            .cloned()
            .map(|v| v.into_inner())
    } else {
        None
    };

    let memory = if let Some(v) = memory {
        v
    } else {
        let memory = ctx
            .services()
            .memories()
            .create(nim::state::MemoryCreate {
                ty: nim::state::MemoryType::Working,
                sources: &[nim::state::Source::Region(nim::state::Region::Audio)],
                summary: caption.value.as_str(),
                confidence: 1.0,
                ..Default::default()
            })
            .await?;

        ctx.signals().publish("nim.memory.create", memory.clone()).await?;
        memory
    };

    ctx.services().artifacts().link(memory.id, signal.content.id).await?;

    let annotations = nim::ai::pipelines::audio::classify::run(std::slice::from_ref(&bytes))?;
    let annotations = annotations.first().map(Vec::as_slice).unwrap_or_default();
    let annotations = annotations.iter().filter(|a| a.score >= 0.2).collect::<Vec<_>>();
    let labels = annotations.iter().map(|a| a.label.to_snake_case()).collect::<Vec<_>>();
    let labels = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let embeddings = nim::ai::pipelines::text::embeddings::run(&labels)?;

    for (i, annotation) in annotations.iter().enumerate() {
        let existing = ctx
            .services()
            .annotations()
            .find_by_artifact_label(signal.content.id, nim::state::Label::Unknown, labels[i])
            .await?;

        if existing.is_some() {
            continue;
        }

        let spans = annotation
            .spans
            .iter()
            .map(|v| nim::state::Span {
                start: v.begin,
                end: v.end,
            })
            .collect::<Vec<_>>();

        let annotation = ctx
            .services()
            .annotations()
            .create(nim::state::AnnotationCreate {
                artifact_id: signal.content.id,
                label: nim::state::Label::Unknown,
                text: labels[i].to_string(),
                spans: &spans,
                embedding: embeddings.get(i).and_then(|a| a.vector.as_deref()),
                confidence: annotation.score as f32,
            })
            .await?;

        ctx.signals().publish("nim.annotation.create", annotation).await?;
    }

    let annotations = ctx.services().annotations().find_by_artifact_id(signal.content.id).await?;
    let confidence = if annotations.is_empty() {
        memory.confidence
    } else {
        annotations.iter().fold(0.0, |acc, v| acc + v.confidence) / annotations.len() as f32
    };
    let memory = ctx
        .services()
        .memories()
        .update(
            memory.id,
            nim::state::MemoryUpdate {
                confidence: Some(confidence),
                ..Default::default()
            },
        )
        .await?;

    ctx.signals().publish("nim.memory.update", memory).await?;
    Ok(())
}
