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

    tracing::debug!("generating captions...");
    let captions = nim::ai::pipelines::audio::captions::run(&[bytes.as_slice()])?;
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

    ctx.signals().publish("nim.audio.artifact.update", artifact).await?;

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
        tracing::debug!("updating existing memory...");
        let mut sources = v.sources;

        if !sources.contains(&nim::state::Source::Region(nim::state::Region::Audio)) {
            sources.push(nim::state::Source::Region(nim::state::Region::Audio));
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
        tracing::debug!("creating new memory...");
        let memory = ctx
            .services()
            .memories()
            .create(nim::state::MemoryCreate {
                ty: nim::state::MemoryType::Working,
                sources: &[nim::state::Source::Region(nim::state::Region::Audio)],
                confidence: 1.0,
                ..Default::default()
            })
            .await?;

        ctx.signals().publish("nim.memory.create", memory.clone()).await?;
        memory
    };

    ctx.ack_progress().await?;
    ctx.services().artifacts().link(memory.id, signal.content.id).await?;

    tracing::debug!("classifying audio...");
    let annotations = nim::ai::pipelines::audio::classify::run_with(
        &[bytes.as_slice()],
        nim::ai::pipelines::audio::classify::Options {
            min_score: 0.2,
            limit: Some(5),
            ..Default::default()
        },
    )?;

    let Some(annotations) = annotations.first() else {
        return Ok(());
    };

    ctx.ack_progress().await?;

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

        if existing.is_some() {
            continue;
        }

        tracing::debug!("creating annotation [{label}] -> {}", annotation.score);
        let spans = annotation
            .spans
            .iter()
            .filter_map(nim::ai::types::Span::as_audio)
            .map(|v| nim::state::Span::Audio {
                start: v.begin,
                end: v.end,
            })
            .collect::<Vec<_>>();

        let annotation = ctx
            .services()
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

        ctx.signals().publish("nim.annotation.create", annotation).await?;
        ctx.ack_progress().await?;
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
