mod error;
mod routes;
mod state;

use tracing_subscriber::EnvFilter;

pub use error::*;
pub use state::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("nim_vision=debug")))
        .init();

    let port: u16 = std::env::var("PORT")?.parse()?;
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = std::sync::Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?,
    );

    sqlx::migrate!("../../crates/nim-store/migrations").run(pool.as_ref()).await?;
    let state = std::sync::Arc::new(State::new(pool));
    let join = tokio::spawn(on_start());
    let app = axum::Router::new().nest("/", routes::new()).with_state(state);
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
