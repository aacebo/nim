use nim::{signals::Signal, state::MemoryRecall};

use crate::{Error, SignalContext};

#[allow(unused_variables)]
#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<MemoryRecall>) -> Result<(), Error> {
    Ok(())
}
