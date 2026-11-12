use nim::{signals::Signal, state::Message};

use crate::{Error, SignalContext};

#[allow(unused)]
#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> Result<(), Error> {
    Ok(())
}
