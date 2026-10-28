use nim::{signals::Signal, state::Memory};

use crate::{Error, SignalContext};

#[allow(unused_variables)]
#[tracing::instrument(skip(ctx))]
pub async fn run(ctx: SignalContext, signal: Signal<Memory>) -> Result<(), Error> {
    Ok(())
}
