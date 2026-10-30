use nim::{signals::Signal, state::MemoryRecall};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<MemoryRecall>) -> Result<(), Error> {
    let memory = ctx.services().memories().find_by_id(signal.content.memory_id).await?;
    let memory = ctx
        .services()
        .memories()
        .update(
            signal.content.memory_id,
            nim::state::MemoryUpdate {
                recalls: Some(memory.recalls + 1),
                recalled_at: Some(chrono::Utc::now()),
                ..Default::default()
            },
        )
        .await?;

    ctx.signals().publish("nim.memory.update", memory).await?;
    Ok(())
}
