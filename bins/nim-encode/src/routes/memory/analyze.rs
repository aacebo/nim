use nim::{signals::Signal, state::Memory};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Memory>) -> Result<(), Error> {
    let artifacts = ctx.services().artifacts().find_by_memory_id(signal.content.id).await?;
    let confidence = artifacts
        .iter()
        .map(|a| {
            if a.confidence.is_finite() {
                a.confidence.clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
        .fold(0.0_f32, f32::max);
    let salience = artifacts
        .iter()
        .map(|a| {
            if a.salience.is_finite() {
                a.salience.clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
        .fold(0.0_f32, f32::max);
    let initial_strength = 1.0 + 6.0 * salience.clamp(0.0, 1.0);
    let summary = artifacts
        .iter()
        .filter_map(|a| a.summary.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let summary = ctx
        .runtime()
        .pipelines()
        .text()
        .summarize(
            &[&summary],
            nim::runtime::ai::TextSummarizeOptions {
                embedding: nim::ai::pipelines::text::embeddings::Options {
                    model: nim::ai::models::refer("unsloth/embeddinggemma-300m")?,
                    embedding: nim::ai::models::EmbedOptions {
                        mode: nim::ai::models::EmbedMode::Document,
                        dimensions: Some(768),
                        ..Default::default()
                    },
                },
                task: nim::runtime::TaskOptions {
                    sources: artifacts.iter().map(|a| a.id.into()).collect(),
                    ..Default::default()
                },
                model: nim::ai::models::refer("Qwen/Qwen2.5-3B-Instruct-GGUF")?,
                ..Default::default()
            },
        )
        .await?;

    let memory = ctx
        .services()
        .memories()
        .update(
            signal.content.id,
            nim::state::MemoryUpdate {
                summary: summary.first().map(|v| v.value.as_str().map(ToString::to_string)).flatten(),
                embedding: summary.first().map(|v| v.vector.as_ref().map(|v| v.clone().into())).flatten(),
                confidence: Some(confidence),
                salience: Some(salience),
                strength: Some(initial_strength),
                ..Default::default()
            },
        )
        .await?;

    ctx.signals().publish("nim.memory.encode", memory).await?;
    Ok(())
}
