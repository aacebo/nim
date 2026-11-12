mod config;
mod context;
mod error;
mod extract;
mod routes;
mod sockets;

pub use config::*;
pub use context::*;
pub use error::*;
pub use sockets::*;

use futures_util::StreamExt;
use tower::Layer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(nim::signals::trace::layer("nim-gateway"))
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nim_gateway", tracing::Level::INFO)
                .with_target("nim_ai", tracing::Level::INFO)
                .with_target("nim_state", tracing::Level::INFO)
                .with_target("nim_signals", tracing::Level::INFO),
        )
        .init();

    let config = Config::load()?;
    let state = std::sync::Arc::new(Context::connect(config).await?);
    let (shutdown, receiver) = tokio::sync::watch::channel(false);

    tokio::select! {
        result = start(state.clone(), receiver) => result,
        result = stop() => {
            let _ = shutdown.send(true);
            Ok(result)
        },
    }
}

#[tracing::instrument(skip_all)]
async fn start(ctx: std::sync::Arc<Context>, mut shutdown: tokio::sync::watch::Receiver<bool>) -> Result<(), Error> {
    let app = axum::Router::new()
        .merge(routes::new())
        .layer(axum::middleware::from_fn_with_state(ctx.clone(), context::http::middleware))
        .with_state(ctx.clone());
    let app = tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash().layer(app);
    let app = axum::ServiceExt::<axum::extract::Request>::into_make_service(app);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", ctx.port)).await?;

    tracing::info!("listening...");

    let events = async move {
        let mut chats = ctx.signals().subscribe("nim.chat.*").await?;
        let mut messages = ctx.signals().subscribe("nim.message.*").await?;
        let mut memories = ctx.signals().subscribe("nim.memory.*").await?;

        loop {
            tokio::select! {
                biased;
                _ = shutdown.changed() => break,
                Some(Ok((message, signal, _span))) = chats.next() => {
                    ctx.sockets().send(message.subject.to_string(), signal).await.map_err(nim::runtime::Error::custom)?;
                },
                Some(Ok((message, signal, _span))) = messages.next() => {
                    ctx.sockets().send(message.subject.to_string(), signal).await.map_err(nim::runtime::Error::custom)?;
                },
                Some(Ok((message, signal, _span))) = memories.next() => {
                    ctx.sockets().send(message.subject.to_string(), signal).await.map_err(nim::runtime::Error::custom)?;
                },
            }
        }

        Ok(())
    };

    tokio::select! {
        result = axum::serve(listener, app).with_graceful_shutdown(stop()) => Ok(result?),
        result = events => result,
    }
}

#[tracing::instrument]
async fn stop() {
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
}
