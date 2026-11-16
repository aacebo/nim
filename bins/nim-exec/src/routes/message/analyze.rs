use nim::{signals::Signal, state::Message};

use crate::SignalContext;

#[allow(unused)]
#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> nim::error::Result<()> {
    Ok(())
}
