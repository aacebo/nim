pub enum Error {
    Sql(nim_store::Error),
}

impl From<nim_store::Error> for Error {
    fn from(value: nim_store::Error) -> Self {
        Self::Sql(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(v) => write!(f, "{v:#?}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sql(v) => Some(v),
        }
    }
}
