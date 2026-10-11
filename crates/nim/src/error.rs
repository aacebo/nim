pub enum Error {
    State(nim_state::Error),
}

impl From<nim_state::Error> for Error {
    fn from(value: nim_state::Error) -> Self {
        Self::State(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(v) => write!(f, "{v:#?}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::State(v) => Some(v),
        }
    }
}
