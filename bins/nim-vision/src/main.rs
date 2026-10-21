mod config;
mod context;
mod error;

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

    let config = Config::load()?;
    let ctx = std::sync::Arc::new(Context::connect(config).await?);
    let service = ctx.signals().service("vision", "0.0.0").await?;
    let mut artifacts = service.endpoint("nim.artifact.>").await?;

    start().await?;

    while let Some(req) = artifacts.next().await {
        let signal = req?.decode::<nim::state::Artifact>()?;
        println!("{signal:#?}");
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
