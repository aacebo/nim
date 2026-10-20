pub enum Error {
    Nats(async_nats::Error),
    Json(serde_json::Error),
}

impl<T> From<async_nats::error::Error<T>> for Error
where
    T: 'static + Send + Sync + Clone + std::fmt::Debug + std::fmt::Display + PartialEq,
{
    fn from(value: async_nats::error::Error<T>) -> Self {
        Self::Nats(Box::new(value))
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nats(err) => write!(f, "{err:#?}"),
            Self::Json(err) => write!(f, "{err:#?}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nats(err) => write!(f, "{err}"),
            Self::Json(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Nats(err) => Some(err.as_ref()),
            Self::Json(err) => Some(err),
        }
    }
}
