pub enum Error {
    AI(nim_ai::Error),
    State(nim_state::Error),
}

impl From<nim_ai::Error> for Error {
    fn from(value: nim_ai::Error) -> Self {
        Self::AI(value)
    }
}

impl From<nim_state::Error> for Error {
    fn from(value: nim_state::Error) -> Self {
        Self::State(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AI(v) => write!(f, "{v:#?}"),
            Self::State(v) => write!(f, "{v:#?}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AI(v) => write!(f, "{v}"),
            Self::State(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AI(v) => Some(v),
            Self::State(v) => Some(v),
        }
    }
}
