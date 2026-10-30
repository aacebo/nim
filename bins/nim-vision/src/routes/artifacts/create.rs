use heck::ToSnakeCase;
use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
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

    tracing::debug!("generating captions...");
    let captions = nim::ai::pipelines::images::captions::run(&[bytes.as_slice()])?;
    let Some(caption) = captions.first() else {
        return Err(Error::custom(format!(
            "no captions could be generated from artifact `{}`",
            signal.content.id
        )));
    };

    ctx.ack_progress().await?;
    let artifact = ctx
        .services()
        .artifacts()
        .update(
            signal.content.id,
            nim::state::ArtifactUpdate {
                summary: caption.value.as_str().map(String::from),
                embedding: caption.vector.as_ref().map(|v| v.clone().into()),
                ..Default::default()
            },
        )
        .await?;

    let memory = if let Some(embedding) = caption.vector.as_deref() {
        ctx.services()
            .memories()
            .search(
                embedding,
                nim::store::MemoryFilter {
                    ty: Some(nim::store::MemoryType::Working),
                    limit: Some(10),
                    min_score: Some(0.2),
                },
            )
            .await?
            .first()
            .cloned()
            .map(|v| v.into_inner())
    } else {
        None
    };

    let (mut memory, subject) = if let Some(v) = memory {
        tracing::debug!("updating existing memory...");
        let mut sources = v.sources;

        if !sources.contains(&nim::state::Source::Region(nim::state::Region::Vision)) {
            sources.push(nim::state::Source::Region(nim::state::Region::Vision));
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

        (memory, "update")
    } else {
        tracing::debug!("creating new memory...");
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

        (memory, "create")
    };

    ctx.ack_progress().await?;
    ctx.services().artifacts().link(memory.id, signal.content.id).await?;

    tracing::debug!("classifying image...");
    let annotations = nim::ai::pipelines::images::classify::run_with(
        &[bytes.as_slice()],
        nim::ai::pipelines::images::classify::Options {
            min_score: 0.2,
            limit: Some(5),
            ..Default::default()
        },
    )?;

    let mut annotations = if let Some(a) = annotations.first() {
        a.clone()
    } else {
        Default::default()
    };

    ctx.ack_progress().await?;

    tracing::debug!("extracting image regions...");
    let regions = nim::ai::pipelines::images::classify::run_with(
        &[bytes.as_slice()],
        nim::ai::pipelines::images::classify::Options {
            model: nim::ai::models::ModelArgs {
                model: Some(String::from("facebook/detr-resnet-50")),
                ..Default::default()
            },
            min_score: 0.2,
            limit: Some(100),
            ..Default::default()
        },
    )?;

    if let Some(regions) = regions.first().cloned() {
        annotations.extend(regions);
    }

    ctx.ack_progress().await?;

    if !annotations.is_empty() {
        let labels = annotations.iter().map(|a| a.label.to_snake_case()).collect::<Vec<_>>();
        let labels = labels.iter().map(String::as_str).collect::<Vec<_>>();
        let input = labels.iter().map(|label| label.as_bytes()).collect::<Vec<_>>();

        tracing::debug!("generating label embeddings...");
        let embeddings = nim::ai::pipelines::text::embeddings::run(&input)?;
        ctx.ack_progress().await?;

        tracing::debug!("generating label categories...");
        let label_annotations = nim::ai::pipelines::text::classify::run_with(
            &input,
            nim::ai::pipelines::text::classify::Options {
                model: nim::ai::models::ModelArgs {
                    model: Some("facebook/bart-large-mnli".into()),
                    ..Default::default()
                },
                labels: Some(vec![
                    nim::state::Label::Entity.as_classify_label().to_string(),
                    nim::state::Label::Valence.as_classify_label().to_string(),
                    nim::state::Label::Emotion.as_classify_label().to_string(),
                    nim::state::Label::Salience.as_classify_label().to_string(),
                    nim::state::Label::Threat.as_classify_label().to_string(),
                    nim::state::Label::Reward.as_classify_label().to_string(),
                    nim::state::Label::Novelty.as_classify_label().to_string(),
                    nim::state::Label::Familiarity.as_classify_label().to_string(),
                    nim::state::Label::Confidence.as_classify_label().to_string(),
                    nim::state::Label::SelfRelevance.as_classify_label().to_string(),
                    nim::state::Label::SocialRelevance.as_classify_label().to_string(),
                    nim::state::Label::Temporal.as_classify_label().to_string(),
                    nim::state::Label::Spatial.as_classify_label().to_string(),
                ]),
                multi_label: true,
                min_score: 0.2,
                limit: None,
                ..Default::default()
            },
        )?;

        ctx.ack_progress().await?;

        for (i, annotation) in annotations.iter().enumerate() {
            let mut label_annotations = label_annotations.get(i).cloned().unwrap_or_default();
            label_annotations.sort_by(|a, b| b.score.total_cmp(&a.score));
            let label = if let Some(first) = label_annotations.first() {
                nim::state::Label::try_from_classify_label(&first.label).unwrap_or_default()
            } else {
                nim::state::Label::Unknown
            };

            let existing = ctx
                .services()
                .annotations()
                .find_by_artifact_label(signal.content.id, label, labels[i])
                .await?;

            if let Some(existing) = &existing
                && existing.confidence as f64 >= annotation.score
            {
                continue;
            }

            tracing::debug!("creating annotation [{label}] -> {}", annotation.score);
            let spans = annotation
                .spans
                .iter()
                .filter_map(nim::ai::types::Span::as_image)
                .map(|v| nim::state::Span::Image {
                    xmin: v.xmin,
                    ymin: v.ymin,
                    xmax: v.xmax,
                    ymax: v.ymax,
                })
                .collect::<Vec<_>>();

            if let Some(existing) = &existing {
                ctx.services()
                    .annotations()
                    .update(
                        existing.id,
                        nim::state::AnnotationUpdate {
                            label: Some(label),
                            text: Some(labels[i].to_string()),
                            spans: Some(spans.clone()),
                            embedding: embeddings.get(i).and_then(|a| a.vector.clone()),
                            confidence: Some(annotation.score as f32),
                        },
                    )
                    .await?;
            } else {
                ctx.services()
                    .annotations()
                    .create(nim::state::AnnotationCreate {
                        artifact_id: signal.content.id,
                        label,
                        text: labels[i].to_string(),
                        spans: &spans,
                        embedding: embeddings.get(i).and_then(|a| a.vector.as_deref()),
                        confidence: annotation.score as f32,
                    })
                    .await?;
            }

            ctx.ack_progress().await?;
        }

        // Calculate Memory Confidence

        let annotations = ctx.services().annotations().find_by_artifact_id(signal.content.id).await?;
        let confidence = annotations.iter().map(|a| a.confidence).max_by(f32::total_cmp);

        memory = ctx
            .services()
            .memories()
            .update(
                memory.id,
                nim::state::MemoryUpdate {
                    confidence,
                    ..Default::default()
                },
            )
            .await?;
    }

    ctx.signals().publish("nim.artifact.create", artifact).await?;
    ctx.signals().publish(format!("nim.memory.{subject}"), memory).await?;
    Ok(())
}
