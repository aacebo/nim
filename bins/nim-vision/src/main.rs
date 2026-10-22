mod config;
mod context;
mod error;
mod routes;

pub use config::*;
pub use context::*;
pub use error::*;

use futures_util::StreamExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nim_vision", tracing::Level::DEBUG)
                .with_target("nim_ai", tracing::Level::DEBUG),
        )
        .init();

    let span = tracing::Span::current();
    let config = Config::load()?;
    let ctx = Context::connect(span, config).await?;
    let mut artifacts = ctx.signals().subscribe("nim.vision.artifact.>").await?;

    start().await?;

    while let Some(req) = artifacts.next().await {
        let (message, signal) = req?;
        let _ = tokio::spawn(routes::artifacts::create::run(ctx.clone().into_signal(message), signal)).await?;
    }

    stop().await
}

#[tracing::instrument]
async fn start() -> Result<(), Error> {
    tracing::info!("listening...");
    Ok(())
}

#[tracing::instrument]
async fn stop() -> Result<(), Error> {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to listen for Ctrl-C");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => tracing::error!(%error, "failed to listen for SIGTERM"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutdown signal received");
    Ok(())
}
