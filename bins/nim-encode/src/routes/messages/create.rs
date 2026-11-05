use heck::ToSnakeCase;
use nim::{signals::Signal, state::Message};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> Result<(), Error> {
    let message = signal.content;
    let content = message.content.to_string();
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .summarize(
            &[content.as_str()],
            nim::runtime::ai::TextSummarizeOptions {
                task: nim::runtime::TaskOptions {
                    sources: vec![message.id.into(), message.chat_id.into()],
                    ..Default::default()
                },
                model: nim::ai::models::refer("Qwen/Qwen2.5-3B-Instruct-GGUF")?,
                ..Default::default()
            },
        )
        .await?;

    if let Some(artifact) = artifacts.first() {
        let message = ctx
            .services()
            .messages()
            .update(
                message.id,
                nim::state::MessageUpdate {
                    summary: artifact.value.as_str().map(String::from),
                    confidence: artifact.confidence.map(|v| v as f32),
                    embedding: artifact.vector.clone(),
                    ..Default::default()
                },
            )
            .await?;

        if let Some(text) = artifact.value.as_str() {
            // Named entities: people, organizations, locations

            tracing::debug!("extracting entities...");

            let annotations = ctx
                .runtime()
                .pipelines()
                .text()
                .classify(
                    &[&text],
                    nim::runtime::ai::TextClassifyOptions {
                        task: nim::runtime::TaskOptions {
                            sources: vec![message.id.into(), message.chat_id.into()],
                            ..Default::default()
                        },
                        model: nim::ai::models::refer("dbmdz/bert-large-cased-finetuned-conll03-english")?,
                        min_score: 0.7,
                        ..Default::default()
                    },
                )
                .await?;

            let mut annotations = annotations.iter().flatten().cloned().collect::<Vec<_>>();

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
                            sources: vec![message.id.into(), message.chat_id.into()],
                            ..Default::default()
                        },
                        model: nim::ai::models::refer("sentence-transformers/all-MiniLM-L6-v2")?,
                        min_score: 0.7,
                        ..Default::default()
                    },
                )
                .await?;

            if let Some(keywords) = keywords.first().cloned() {
                annotations.extend(keywords);
            }

            ctx.ack_progress().await?;

            if !annotations.is_empty() {
                let labels = annotations.iter().map(|a| a.label.to_snake_case()).collect::<Vec<_>>();
                let labels = labels.iter().map(String::as_str).collect::<Vec<_>>();

                tracing::debug!("generating label embeddings...");
                let embeddings = ctx
                    .runtime()
                    .pipelines()
                    .text()
                    .embeddings(
                        &labels,
                        nim::runtime::ai::TextEmbeddingsOptions {
                            task: nim::runtime::TaskOptions {
                                sources: vec![message.id.into(), message.chat_id.into()],
                                ..Default::default()
                            },
                            model: nim::ai::models::refer("sentence-transformers/all-MiniLM-L12-v2")?,
                        },
                    )
                    .await?;

                ctx.ack_progress().await?;

                tracing::debug!("generating label categories...");
                let label_annotations = ctx
                    .runtime()
                    .pipelines()
                    .text()
                    .classify(
                        &labels,
                        nim::runtime::ai::TextClassifyOptions {
                            task: nim::runtime::TaskOptions {
                                sources: vec![message.id.into(), message.chat_id.into()],
                                ..Default::default()
                            },
                            model: nim::ai::models::refer("facebook/bart-large-mnli")?,
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
                            ..Default::default()
                        },
                    )
                    .await?;

                ctx.ack_progress().await?;

                for (i, annotation) in annotations.iter().enumerate() {
                    let mut label_annotations = label_annotations[i].clone();
                    label_annotations.sort_by(|a, b| b.score.total_cmp(&a.score));
                    let label = if let Some(first) = label_annotations.first() {
                        nim::state::Label::try_from_classify_label(&first.label).unwrap_or_default()
                    } else {
                        nim::state::Label::Unknown
                    };

                    let existing = ctx
                        .services()
                        .messages()
                        .find_annotation_by_label(message.id, label, labels[i])
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
                        .filter_map(nim::ai::types::Span::as_text)
                        .map(|v| nim::state::Span::Text {
                            start: v.begin,
                            end: v.end,
                        })
                        .collect::<Vec<_>>();

                    if let Some(existing) = &existing {
                        ctx.services()
                            .messages()
                            .update_annotation(
                                message.id,
                                existing.id,
                                nim::state::MessageAnnotationUpdate {
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
                            .messages()
                            .create_annotation(
                                message.id,
                                nim::state::MessageAnnotationCreate {
                                    label,
                                    text: labels[i].to_string(),
                                    spans: &spans,
                                    embedding: embeddings.get(i).and_then(|a| a.vector.as_deref()),
                                    confidence: annotation.score as f32,
                                },
                            )
                            .await?;
                    }

                    ctx.ack_progress().await?;
                }
            }
        }

        ctx.signals().publish("nim.message.update", message.clone()).await?;
        ctx.signals().publish("nim.message.analyze", message).await?;
    }

    Ok(())
}
