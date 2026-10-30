use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    let annotations = ctx.services().annotations().find_by_artifact_id(signal.content.id).await?;
    let confidence = annotations
        .iter()
        .map(|a| {
            if a.confidence.is_finite() {
                a.confidence.clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
        .fold(0.0_f32, f32::max);
    let salience = annotations
        .iter()
        .map(|a| {
            let confidence = if a.confidence.is_finite() {
                a.confidence.clamp(0.0, 1.0)
            } else {
                0.0
            };

            confidence * a.label.label_multiplier()
        })
        .fold(0.0_f32, f32::max);

    let initial_strength = 1.0 + 6.0 * salience.clamp(0.0, 1.0);

    ctx.services()
        .artifacts()
        .update(
            signal.content.id,
            nim::state::ArtifactUpdate {
                confidence: Some(confidence),
                salience: Some(salience),
                strength: Some(initial_strength),
                ..Default::default()
            },
        )
        .await?;

    let memories = ctx.services().memories().find_by_artifact_id(signal.content.id).await?;

    for memory in memories {
        let artifacts = ctx.services().artifacts().find_by_memory_id(memory.id).await?;
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

        ctx.services()
            .memories()
            .update(
                memory.id,
                nim::state::MemoryUpdate {
                    confidence: Some(confidence),
                    salience: Some(salience),
                    strength: Some(initial_strength),
                    ..Default::default()
                },
            )
            .await?;
    }

    Ok(())
}
