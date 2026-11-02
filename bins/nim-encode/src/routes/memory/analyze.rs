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

    ctx.services()
        .memories()
        .update(
            signal.content.id,
            nim::state::MemoryUpdate {
                confidence: Some(confidence),
                salience: Some(salience),
                strength: Some(initial_strength),
                ..Default::default()
            },
        )
        .await?;

    Ok(())
}
