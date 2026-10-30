use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[allow(unused_variables)]
#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    let annotations = ctx.services().annotations().find_by_artifact_id(signal.content.id).await?;
    let confidence = annotations.iter().map(|a| a.confidence).max_by(f32::total_cmp);
    let salience = annotations
        .iter()
        .map(|a| {
            let weight = a.label.label_multiplier();
            a.confidence.clamp(0.0, 1.0) * weight
        })
        .fold(0.0_f32, f32::max);
    let strength = 1.0 + 6.0 * salience.clamp(0.0, 1.0);
    Ok(())
}
