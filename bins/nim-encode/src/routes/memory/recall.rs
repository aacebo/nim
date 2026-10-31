use nim::{signals::Signal, state::MemoryRecall};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<MemoryRecall>) -> Result<(), Error> {
    let memory = ctx.services().memories().find_by_id(signal.content.memory_id).await?;
    let now = chrono::Utc::now();
    let last_recalled_at = memory.recalled_at.unwrap_or(memory.created_at);
    let elapsed = (now - last_recalled_at).max(chrono::TimeDelta::zero());
    let artifacts = ctx.services().artifacts().find_by_memory_id(signal.content.memory_id).await?;
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

    let reinforcement = 0.2;
    let quality = if signal.content.score.is_finite() {
        f64::from(signal.content.score).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let strength = recall_strength(
        memory.strength as f64,
        elapsed,
        Some(ctx.strength_half_life()),
        reinforcement,
        quality,
    );

    let memory = ctx
        .services()
        .memories()
        .update(
            signal.content.memory_id,
            nim::state::MemoryUpdate {
                confidence: Some(confidence),
                strength: strength.map(|v| v as f32),
                recalls: Some(memory.recalls + 1),
                recalled_at: Some(now),
                ..Default::default()
            },
        )
        .await?;

    ctx.signals().publish("nim.memory.update", memory).await?;
    Ok(())
}

fn recall_strength(
    strength: f64,
    elapsed: chrono::TimeDelta,
    half_life: Option<chrono::TimeDelta>,
    reinforcement: f64,
    quality: f64,
) -> Option<f64> {
    if !strength.is_finite()
        || !reinforcement.is_finite()
        || !quality.is_finite()
        || !(0.0..=7.0).contains(&strength)
        || !(0.0..=1.0).contains(&reinforcement)
        || !(0.0..=1.0).contains(&quality)
        || elapsed < chrono::TimeDelta::zero()
    {
        return None;
    }

    let decay = match half_life {
        Some(value) if value > chrono::TimeDelta::zero() => 2.0_f64.powf(-elapsed.as_seconds_f64() / value.as_seconds_f64()),
        Some(_) => return None,
        None => 1.0,
    };

    let decayed_strength = strength * decay;
    let boost = reinforcement * quality * (7.0 - decayed_strength);

    Some((decayed_strength + boost).clamp(0.0, 7.0))
}
