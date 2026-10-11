#[derive(Debug)]
pub enum Error {
    Core(nim::Error),
    IO(std::io::Error),
    Http(axum::response::Response),
    Env(&'static str, std::env::VarError),
    ParseInt(std::num::ParseIntError),
    Migrate(sqlx::migrate::MigrateError),
    Join(tokio::task::JoinError),
    Multipart(axum::extract::multipart::MultipartError),
}

impl From<nim::Error> for Error {
    fn from(value: nim::Error) -> Self {
        Self::Core(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<(&'static str, std::env::VarError)> for Error {
    fn from((key, err): (&'static str, std::env::VarError)) -> Self {
        Self::Env(key, err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParseInt(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Core(nim::Error::State(value.into()))
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::Migrate(value)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Join(value)
    }
}

impl From<axum::extract::multipart::MultipartError> for Error {
    fn from(value: axum::extract::multipart::MultipartError) -> Self {
        Self::Multipart(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(err) => write!(f, "{err}"),
            Self::Http(err) => write!(
                f,
                "[error::http::{}] -> {}",
                err.status(),
                err.status().canonical_reason().unwrap_or("??")
            ),
            Self::Env(key, err) => write!(f, "[error::env::{key}] -> {err}"),
            Self::ParseInt(err) => write!(f, "[error::int::parse] -> {err}"),
            Self::Migrate(err) => write!(f, "[error::migrate] -> {err}"),
            Self::IO(err) => write!(f, "[error::io] -> {err}"),
            Self::Join(err) => write!(f, "[error::json] -> {err}"),
            Self::Multipart(err) => write!(f, "[error::http::multipart] -> {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(err) => Some(err),
            Self::Env(_, err) => Some(err),
            Self::ParseInt(err) => Some(err),
            Self::Migrate(err) => Some(err),
            Self::IO(err) => Some(err),
            Self::Multipart(err) => Some(err),
            _ => None,
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Http(err) => err,
            Self::Multipart(err) => err.into_response(),
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
