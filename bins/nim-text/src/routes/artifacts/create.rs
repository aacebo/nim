use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(_ctx))]
pub async fn run(_ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    Ok(())
}
