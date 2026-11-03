mod config;
mod context;
mod error;
mod routes;

pub use config::*;
pub use context::*;
pub use error::*;

use futures_util::StreamExt;
use nim::signals::ToSignal;
use tracing::Instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(nim::signals::trace::layer("nim-text"))
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("nim_text", tracing::Level::INFO)
                .with_target("nim_ai", tracing::Level::INFO)
                .with_target("nim_state", tracing::Level::INFO)
                .with_target("nim_signals", tracing::Level::INFO),
        )
        .init();

    let span = tracing::Span::current();
    let config = Config::load()?;
    let ctx = Context::connect(span, config).await?;

    start().await?;

    let (shutdown, receiver) = tokio::sync::watch::channel(false);
    let mut worker = tokio::spawn(artifacts(ctx, receiver));

    tokio::select! {
        result = &mut worker => result?,
        result = stop() => {
            let _ = shutdown.send(true);
            worker.await??;
            result
        }
    }
}

#[tracing::instrument(skip(ctx, shutdown))]
async fn artifacts(ctx: Context, mut shutdown: tokio::sync::watch::Receiver<bool>) -> Result<(), Error> {
    let mut artifacts = ctx.signals().subscribe("nim.text.artifact.create").await?;

    loop {
        let req = tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            req = artifacts.next() => req,
        };

        let Some(req) = req else {
            break;
        };

        let (message, signal, span) = req?;
        let res: Result<(), Error> = async {
            let subject = message.subject.to_string();
            let ctx = ctx.clone().into_signal(signal.id, message);

            tracing::info!("[{subject}] -> processing...");

            let res = tokio::spawn(routes::artifacts::create::run(ctx.clone(), signal.to_signal()?).in_current_span()).await?;

            match res {
                Ok(_) => {
                    tracing::info!("[{subject}.success] -> complete");
                    ctx.ack().await?
                }
                Err(err) => {
                    tracing::Span::current().record("otel.status_code", "ERROR");
                    tracing::error!("[{subject}.error] -> {err}");
                    ctx.nack().await?
                }
            };

            Ok(())
        }
        .instrument(span.clone())
        .await;

        if res.is_err() {
            span.record("otel.status_code", "ERROR");
        }

        res?;
    }

    Ok(())
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
