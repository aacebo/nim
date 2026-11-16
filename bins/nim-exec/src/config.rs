#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
}

impl Config {
    #[tracing::instrument]
    pub fn load() -> nim::error::Result<Self> {
        Ok(Self {
            port: std::env::var("PORT").unwrap_or("8080".into()).parse()?,
            database_url: std::env::var("DATABASE_URL")
                .map_err(|err| nim::error::Diagnostic::message(format!("DATABASE_URL: {err}")))?,
            nats_url: std::env::var("NATS_URL").map_err(|err| nim::error::Diagnostic::message(format!("NATS_URL: {err}")))?,
        })
    }
}
