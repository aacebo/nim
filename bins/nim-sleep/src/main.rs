mod config;
mod context;

pub use config::*;
pub use context::*;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> nim::error::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(nim::signals::trace::layer("nim-sleep"))
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nim_sleep", tracing::Level::INFO)
                .with_target("nim_ai", tracing::Level::INFO)
                .with_target("nim_state", tracing::Level::INFO)
                .with_target("nim_signals", tracing::Level::INFO),
        )
        .init();

    let span = tracing::Span::current();
    let config = Config::load()?;
    let _ctx = Context::connect(span, config).await?;

    start().await?;

    // Do Stuff...

    stop().await
}

#[tracing::instrument]
async fn start() -> nim::error::Result<()> {
    tracing::info!("started");
    Ok(())
}

#[tracing::instrument]
async fn stop() -> nim::error::Result<()> {
    tracing::info!("complete");
    Ok(())
}
