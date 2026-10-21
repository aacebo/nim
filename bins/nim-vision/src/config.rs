use crate::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
    pub staging_dir: std::path::PathBuf,
}

impl Config {
    #[tracing::instrument]
    pub fn load() -> Result<Self, Error> {
        use std::str::FromStr;

        let staging_dir = std::env::var("STAGING_DIR").unwrap_or(".staging".into());

        Ok(Self {
            port: std::env::var("PORT").unwrap_or("8080".into()).parse()?,
            database_url: std::env::var("DATABASE_URL").map_err(|err| ("DATABASE_URL", err))?,
            nats_url: std::env::var("NATS_URL").map_err(|err| ("NATS_URL", err))?,
            staging_dir: std::path::PathBuf::from_str(&staging_dir)?,
        })
    }
}
