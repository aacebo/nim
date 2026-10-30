use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[allow(unused_variables)]
#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    // let salience =
    Ok(())
}
