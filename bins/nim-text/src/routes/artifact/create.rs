use heck::ToSnakeCase;
use nim::{signals::Signal, state::Artifact};

use crate::SignalContext;

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> nim::error::Result<()> {
    let text = match &signal.content.content {
        nim::state::ArtifactContent::Text { text } => text.to_lowercase(),
        nim::state::ArtifactContent::File {
            path,
            size: _,
            media_type: _,
        } => tokio::fs::read_to_string(path).await?.to_lowercase(),
        nim::state::ArtifactContent::Resource { url, headers, .. } => {
            let mut request = reqwest::Client::new().get(url);

            for (name, value) in headers {
                request = request.header(name, value);
            }

            request.send().await?.error_for_status()?.text().await?.to_lowercase()
        }
    };

    tracing::debug!("summarizing...");
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .summarize(
            &[&text],
            nim::runtime::ai::TextSummarizeOptions {
                embedding: nim::ai::pipelines::text::embeddings::Options {
                    model: nim::ai::models::refer("unsloth/embeddinggemma-300m")?,
                    embedding: nim::ai::EmbedOptions {
                        mode: nim::ai::EmbedMode::Document,
                        dimensions: Some(768),
                        ..Default::default()
                    },
                },
                task: nim::runtime::TaskOptions {
                    sources: vec![signal.content.id.into()],
                    ..Default::default()
                },
                model: nim::ai::models::refer("Qwen/Qwen2.5-1.5B-Instruct-GGUF")?,
                ..Default::default()
            },
        )
        .await?;

    let Some(summary) = artifacts.first() else {
        return Err(nim::error::Diagnostic::message(format!(
            "summary could not be generated from artifact `{}`",
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
                summary: summary.value.as_str().map(String::from),
                embedding: summary.vector.as_ref().map(|v| v.clone().into()),
                ..Default::default()
            },
        )
        .await?;

    let memory = if let Some(embedding) = summary.vector.as_deref() {
        ctx.services()
            .search()
            .memories(
                embedding,
                nim::store::MemoryFilter {
                    ty: Some(nim::state::MemoryType::Working.to_string()),
                    limit: Some(10),
                    min_score: Some(0.2),
                    ..Default::default()
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

        (memory, "update")
    } else {
        tracing::debug!("creating new memory...");
        let memory = ctx
            .services()
            .memories()
            .create(nim::state::MemoryCreate {
                ty: nim::state::MemoryType::Working,
                sources: &[nim::state::Source::Region(nim::state::Region::Text)],
                ..Default::default()
            })
            .await?;

        (memory, "create")
    };

    ctx.ack_progress().await?;
    ctx.services().artifacts().link(memory.id, signal.content.id).await?;

    // Named entities: people, organizations, locations

    tracing::debug!("extracting entities...");

    let classifications = ctx
        .runtime()
        .pipelines()
        .text()
        .classify(
            &[&text],
            nim::runtime::ai::TextClassifyOptions {
                task: nim::runtime::TaskOptions {
                    sources: vec![signal.content.id.into()],
                    ..Default::default()
                },
                model: nim::ai::models::refer("dbmdz/bert-base-cased-finetuned-conll03-english")?,
                min_score: 0.2,
                ..Default::default()
            },
        )
        .await?;

    let mut classifications = classifications.iter().flatten().cloned().collect::<Vec<_>>();

    ctx.ack_progress().await?;
    tracing::debug!("extracting keywords...");

    let keywords = ctx
        .runtime()
        .pipelines()
        .text()
        .classify(
            &[&text],
            nim::runtime::ai::TextClassifyOptions {
                task: nim::runtime::TaskOptions {
                    sources: vec![signal.content.id.into()],
                    ..Default::default()
                },
                model: nim::ai::models::refer("sentence-transformers/all-MiniLM-L6-v2")?,
                min_score: 0.2,
                ..Default::default()
            },
        )
        .await?;

    if let Some(keywords) = keywords.first().cloned() {
        classifications.extend(keywords);
    }

    ctx.ack_progress().await?;

    if !classifications.is_empty() {
        let labels = classifications.iter().map(|a| a.label.to_snake_case()).collect::<Vec<_>>();
        let labels = labels.iter().map(String::as_str).collect::<Vec<_>>();

        tracing::debug!("generating label embeddings...");
        let embeddings = ctx
            .runtime()
            .pipelines()
            .text()
            .embed(
                &labels,
                nim::runtime::ai::TextEmbedOptions {
                    embedding: nim::ai::EmbedOptions {
                        mode: nim::ai::EmbedMode::Document,
                        dimensions: Some(768),
                        ..Default::default()
                    },
                    task: nim::runtime::TaskOptions {
                        sources: vec![signal.content.id.into()],
                        ..Default::default()
                    },
                    model: nim::ai::models::refer("unsloth/embeddinggemma-300m")?,
                },
            )
            .await?;

        ctx.ack_progress().await?;

        tracing::debug!("generating label categories...");
        let label_classifications = ctx
            .runtime()
            .pipelines()
            .text()
            .classify(
                &labels,
                nim::runtime::ai::TextClassifyOptions {
                    task: nim::runtime::TaskOptions {
                        sources: vec![signal.content.id.into()],
                        ..Default::default()
                    },
                    model: nim::ai::models::refer("valhalla/distilbart-mnli-12-3")?,
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
                    ..Default::default()
                },
            )
            .await?;

        ctx.ack_progress().await?;

        for (i, classification) in classifications.iter().enumerate() {
            let mut label_classifications = label_classifications[i].clone();
            label_classifications.sort_by(|a, b| b.score.total_cmp(&a.score));
            let label = if let Some(first) = label_classifications.first() {
                nim::state::Label::try_from_classify_label(&first.label).unwrap_or_default()
            } else {
                nim::state::Label::Unknown
            };

            let existing = ctx
                .services()
                .artifacts()
                .find_annotation_by_label(signal.content.id, label, labels[i])
                .await?;

            if let Some(existing) = &existing
                && existing.confidence as f64 >= classification.score
            {
                continue;
            }

            tracing::debug!("creating annotation [{label}] -> {}", classification.score);
            let spans = classification
                .spans
                .iter()
                .filter_map(nim::ai::types::Span::as_offset)
                .map(|v| nim::state::Span::Text {
                    start: v.begin,
                    end: v.end,
                })
                .collect::<Vec<_>>();

            if let Some(existing) = &existing {
                ctx.services()
                    .artifacts()
                    .update_annotation(
                        signal.content.id,
                        existing.id,
                        nim::state::ArtifactAnnotationUpdate {
                            label: Some(label),
                            text: Some(labels[i].to_string()),
                            spans: Some(spans.clone()),
                            embedding: embeddings.get(i).and_then(|a| a.vector.clone()),
                            confidence: Some(classification.score as f32),
                        },
                    )
                    .await?;
            } else {
                ctx.services()
                    .artifacts()
                    .create_annotation(
                        signal.content.id,
                        nim::state::ArtifactAnnotationCreate {
                            label,
                            text: labels[i].to_string(),
                            spans: &spans,
                            embedding: embeddings.get(i).and_then(|a| a.vector.as_deref()),
                            confidence: classification.score as f32,
                        },
                    )
                    .await?;
            }

            ctx.ack_progress().await?;
        }

        // Calculate Memory Confidence

        let annotations = ctx.services().artifacts().find_annotations(signal.content.id).await?;
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

    ctx.signals().publish("nim.artifact.create", artifact.clone()).await?;
    ctx.signals().publish(format!("nim.memory.{subject}"), memory).await?;
    ctx.signals().publish("nim.artifact.analyze", artifact).await?;
    Ok(())
}
