use crate::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
}

impl Config {
    #[tracing::instrument]
    pub fn load() -> Result<Self, Error> {
        Ok(Self {
            port: std::env::var("PORT").unwrap_or("8080".into()).parse()?,
            database_url: std::env::var("DATABASE_URL").map_err(|err| ("DATABASE_URL", err))?,
            nats_url: std::env::var("NATS_URL").map_err(|err| ("NATS_URL", err))?,
        })
    }
}
