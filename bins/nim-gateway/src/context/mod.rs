use crate::{Config, Error};

pub mod http;

#[derive(Clone)]
pub struct Context {
    pool: std::sync::Arc<sqlx::PgPool>,
    signals: std::sync::Arc<nim::signals::Client>,
}

impl Context {
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>, signals: std::sync::Arc<nim::signals::Client>) -> Self {
        Self { pool, signals }
    }

    pub fn services(&self) -> nim::state::Services<'_> {
        nim::state::Services::new(&self.pool)
    }

    pub fn signals(&self) -> &nim::signals::Client {
        &self.signals
    }
}

impl Context {
    pub async fn connect(config: &Config) -> Result<Self, Error> {
        let pool = std::sync::Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?,
        );

        sqlx::migrate!("../../crates/nim-store/migrations").run(pool.as_ref()).await?;
        let signals = nim::signals::connect("gateway", &config.nats_url).await?;
        let signals = std::sync::Arc::new(signals);
        Ok(Self { pool, signals })
    }
}
