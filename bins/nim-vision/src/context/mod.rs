mod signal;

pub use signal::*;

use crate::{Config, Error};

#[derive(Clone)]
pub struct Context {
    span: tracing::Span,
    config: Config,
    pool: std::sync::Arc<sqlx::PgPool>,
    signals: std::sync::Arc<nim::signals::Client>,
    _models: std::sync::Arc<nim::ai::Subscription>,
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

    pub fn into_signal(self, id: uuid::Uuid, message: nim::signals::Message) -> SignalContext {
        SignalContext::new(self, id, message)
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
        let signals = nim::signals::connect("vision", &config.nats_url).await?;
        let signals = std::sync::Arc::new(signals);
        let models = std::sync::Arc::new(nim::runtime::models::persist(pool.as_ref().clone()).map_err(nim::Error::from)?);

        Ok(Self {
            span,
            config,
            pool,
            signals,
            _models: models,
        })
    }
}

impl std::ops::Deref for Context {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
