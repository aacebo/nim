use crate::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub nats_url: String,
    pub strength_half_life: chrono::TimeDelta,
}

impl Config {
    #[tracing::instrument]
    pub fn load() -> Result<Self, Error> {
        let days = match std::env::var("STRENGTH_HALF_LIFE_DAYS") {
            Ok(value) => value.parse::<f64>().map_err(Error::custom)?,
            Err(std::env::VarError::NotPresent) => 30.0,
            Err(err) => return Err(("STRENGTH_HALF_LIFE_DAYS", err).into()),
        };

        let strength_half_life = std::time::Duration::try_from_secs_f64(days * 86400.0)
            .ok()
            .and_then(|duration| chrono::TimeDelta::from_std(duration).ok())
            .filter(|duration| *duration > chrono::TimeDelta::zero())
            .ok_or_else(|| Error::custom("STRENGTH_HALF_LIFE_DAYS must be positive, finite, and representable as a duration"))?;

        Ok(Self {
            strength_half_life,
            port: std::env::var("PORT").unwrap_or("8080".into()).parse()?,
            database_url: std::env::var("DATABASE_URL").map_err(|err| ("DATABASE_URL", err))?,
            nats_url: std::env::var("NATS_URL").map_err(|err| ("NATS_URL", err))?,
        })
    }
}
