use crate::{Config, Error};

pub mod http;

#[derive(Clone)]
pub struct Context {
    config: Config,
    pool: std::sync::Arc<sqlx::PgPool>,
    signals: std::sync::Arc<nim::signals::Client>,
}

impl Context {
    pub fn services(&self) -> nim::state::Services<'_> {
        nim::state::Services::new(&self.pool)
    }

    pub fn signals(&self) -> &nim::signals::Client {
        &self.signals
    }
}

impl Context {
    #[tracing::instrument]
    pub async fn connect(config: Config) -> Result<Self, Error> {
        let pool = std::sync::Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?,
        );

        sqlx::migrate!("../../crates/nim-store/migrations").run(pool.as_ref()).await?;
        let signals = nim::signals::connect("gateway", &config.nats_url).await?;
        let signals = std::sync::Arc::new(signals);
        Ok(Self { config, pool, signals })
    }
}

impl std::ops::Deref for Context {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
