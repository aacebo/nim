mod config;
mod context;
mod error;
mod routes;

pub use config::*;
pub use context::*;
pub use error::*;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nim_gateway", tracing::Level::DEBUG)
                .with_target("nim_ai", tracing::Level::DEBUG),
        )
        .init();

    let config = Config::load()?;
    let state = std::sync::Arc::new(Context::connect(&config).await?);
    let join = tokio::spawn(start());
    let app = axum::Router::new()
        .merge(routes::new())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(axum::middleware::from_fn_with_state(state.clone(), context::http::middleware))
        .with_state(state);
    let app = axum::ServiceExt::<axum::extract::Request>::into_make_service(app);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;

    axum::serve(listener, app).with_graceful_shutdown(stop()).await?;
    let _ = join.await?;
    Ok(())
}

#[tracing::instrument]
async fn start() -> Result<(), nim::Error> {
    tracing::info!("listening...");
    Ok(())
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
