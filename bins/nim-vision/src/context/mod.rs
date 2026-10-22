mod signal;

pub use signal::*;

use crate::{Config, Error};

#[derive(Clone)]
pub struct Context {
    span: tracing::Span,
    config: Config,
    pool: std::sync::Arc<sqlx::PgPool>,
    signals: std::sync::Arc<nim::signals::Client>,
}

impl Context {
    pub fn span(&self) -> &tracing::Span {
        &self.span
    }

    pub fn services(&self) -> nim::state::Services<'_> {
        nim::state::Services::new(&self.pool)
    }

    pub fn signals(&self) -> &nim::signals::Client {
        &self.signals
    }

    pub fn into_signal(self, message: nim::signals::Message) -> SignalContext {
        SignalContext::new(self, message)
    }
}

impl Context {
    #[tracing::instrument]
    pub async fn connect(span: tracing::Span, config: Config) -> Result<Self, Error> {
        let pool = std::sync::Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?,
        );

        sqlx::migrate!("../../crates/nim-store/migrations").run(pool.as_ref()).await?;
        let signals = nim::signals::connect_with(
            "vision",
            &config.nats_url,
            nim::signals::ConnectOptions {
                streams: vec![
                    nim::signals::StreamConfig {
                        name: "ARTIFACTS".into(),
                        subjects: vec![
                            "nim.vision.artifact.create".into(),
                            "nim.audio.artifact.create".into(),
                            "nim.unknown.artifact.create".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "MEMORIES".into(),
                        subjects: vec![
                            "nim.memory.create".into(),
                            "nim.memory.update".into(),
                            "nim.memory.delete".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "ENTITIES".into(),
                        subjects: vec![
                            "nim.entity.create".into(),
                            "nim.entity.update".into(),
                            "nim.entity.delete".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "ANNOTATIONS".into(),
                        subjects: vec![
                            "nim.annotation.create".into(),
                            "nim.annotation.update".into(),
                            "nim.annotation.delete".into(),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )
        .await?;

        let signals = std::sync::Arc::new(signals);
        Ok(Self {
            span,
            config,
            pool,
            signals,
        })
    }
}

impl std::ops::Deref for Context {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
