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

    // Extract Captions (ex. "a dog running")

    let captions = nim::ai::pipelines::images::captions::run(std::slice::from_ref(&bytes))?;
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
                sources: &[nim::state::Source::Region(nim::state::Region::Vision)],
                summary: caption.value.as_str(),
                confidence: 1.0,
                ..Default::default()
            })
            .await?;

        ctx.signals().publish("nim.memory.create", memory.clone()).await?;
        memory
    };

    ctx.services().artifacts().link(memory.id, signal.content.id).await?;

    // Extract Annotations (ex. "golden retriever")

    let annotations = nim::ai::pipelines::images::classify::run(std::slice::from_ref(&bytes))?;
    let Some(annotations) = annotations.first() else {
        return Ok(());
    };

    let annotations = annotations.iter().filter(|a| a.score >= 0.2).collect::<Vec<_>>();
    let labels = annotations.iter().map(|a| a.label.as_str()).collect::<Vec<_>>();
    let embeddings = nim::ai::pipelines::text::embeddings::run(&labels)?;

    for (i, annotation) in annotations.iter().enumerate() {
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
                memory_id: memory.id,
                label: nim::state::Label::Unknown,
                text: annotation.label.clone(),
                spans: &spans,
                embedding: embeddings.get(i).and_then(|a| a.vector.as_deref()),
                confidence: annotation.score as f32,
            })
            .await?;

        ctx.signals().publish("nim.annotation.create", annotation).await?;
    }

    // Extract Broader Annotations (ex. "dog")

    let detections = nim::ai::pipelines::images::detect::run(std::slice::from_ref(&bytes))?;
    let Some(detections) = detections.first() else {
        return Ok(());
    };

    let detections = detections.iter().filter(|a| a.score >= 0.2).collect::<Vec<_>>();
    let labels = annotations.iter().map(|a| a.label.as_str()).collect::<Vec<_>>();
    let embeddings = nim::ai::pipelines::text::embeddings::run(&labels)?;

    for (i, detection) in detections.iter().enumerate() {
        let annotation = ctx
            .services()
            .annotations()
            .create(nim::state::AnnotationCreate {
                memory_id: memory.id,
                label: nim::state::Label::Unknown,
                text: detection.label.clone(),
                spans: Default::default(),
                embedding: embeddings.get(i).and_then(|a| a.vector.as_deref()),
                confidence: detection.score as f32,
            })
            .await?;

        ctx.signals().publish("nim.annotation.create", annotation).await?;
    }

    // Calculate Memory Confidence

    let annotations = ctx.services().annotations().find_by_memory_id(memory.id).await?;
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
