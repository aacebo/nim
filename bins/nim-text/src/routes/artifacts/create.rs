use heck::ToSnakeCase;
use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    let text = match &signal.content.content {
        nim::state::ArtifactContent::Text { text } => text.clone(),
        nim::state::ArtifactContent::File {
            path,
            size: _,
            media_type: _,
        } => tokio::fs::read_to_string(path).await?,
    };

    let artifacts = nim::ai::pipelines::text::summarize::run(&[&text])?;
    let Some(summary) = artifacts.first() else {
        return Err(Error::custom(format!(
            "summary could not be generated from artifact `{}`",
            signal.content.id
        )));
    };

    let artifact = ctx
        .services()
        .artifacts()
        .update(
            signal.content.id,
            nim::state::ArtifactUpdate {
                summary: summary.value.as_str().map(String::from),
                embedding: summary.vector.as_ref().map(|v| v.clone().into()),
                ..Default::default()
            },
        )
        .await?;

    ctx.signals().publish("nim.text.artifact.update", artifact).await?;

    let memory = if let Some(embedding) = summary.vector.as_deref() {
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
        let mut sources = v.sources;

        if !sources.contains(&nim::state::Source::Region(nim::state::Region::Text)) {
            sources.push(nim::state::Source::Region(nim::state::Region::Text));
        }

        let memory = ctx
            .services()
            .memories()
            .update(
                v.id,
                nim::state::MemoryUpdate {
                    sources: Some(&sources),
                    ..Default::default()
                },
            )
            .await?;

        ctx.signals().publish("nim.memory.update", memory.clone()).await?;
        memory
    } else {
        let memory = ctx
            .services()
            .memories()
            .create(nim::state::MemoryCreate {
                ty: nim::state::MemoryType::Working,
                sources: &[nim::state::Source::Region(nim::state::Region::Vision)],
                confidence: 1.0,
                ..Default::default()
            })
            .await?;

        ctx.signals().publish("nim.memory.create", memory.clone()).await?;
        memory
    };

    ctx.services().artifacts().link(memory.id, signal.content.id).await?;

    // Extract Entities

    let annotations = nim::ai::pipelines::text::classify::entities(std::slice::from_ref(&text.as_str()))?;
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

    // Extract Keywords

    let annotations = nim::ai::pipelines::text::classify::keywords(std::slice::from_ref(&text.as_str()))?;
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

    // Extract PII

    let annotations = nim::ai::pipelines::text::classify::pii(std::slice::from_ref(&text.as_str()))?;
    let Some(annotations) = annotations.first() else {
        return Ok(());
    };

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
            .map(|span| nim::state::Span {
                start: span.begin,
                end: span.end,
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

    // Calculate Memory Confidence

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
