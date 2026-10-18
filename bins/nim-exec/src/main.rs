mod error;
mod state;

pub use error::*;
pub use state::*;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nim_exec", tracing::Level::DEBUG)
                .with_target("nim_ai", tracing::Level::DEBUG),
        )
        .init();

    let port: u16 = std::env::var("PORT").unwrap_or("8080".into()).parse()?;
    let database_url = std::env::var("DATABASE_URL").map_err(|err| ("DATABASE_URL", err))?;
    let pool = std::sync::Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?,
    );

    sqlx::migrate!("../../crates/nim-store/migrations").run(pool.as_ref()).await?;
    let state = std::sync::Arc::new(State::new(pool));
    let join = tokio::spawn(on_start());
    let app = axum::Router::new().with_state(state);
    let app = axum::ServiceExt::<axum::extract::Request>::into_make_service(app);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    axum::serve(listener, app).with_graceful_shutdown(on_stop()).await?;
    let _ = join.await?;
    Ok(())
}

#[tracing::instrument]
async fn on_start() -> Result<(), nim::Error> {
    tracing::info!("listening...");
    Ok(())
}

#[tracing::instrument]
async fn on_stop() {
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
