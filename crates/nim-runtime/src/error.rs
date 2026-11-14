pub enum Error {
    Custom(String),
    IO(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    AI(nim_ai::Error),
    State(nim_state::Error),
    Signal(nim_signals::Error),
}

impl Error {
    pub fn custom(message: impl std::fmt::Display) -> Self {
        Self::Custom(message.to_string())
    }
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

impl From<nim_signals::Error> for Error {
    fn from(value: nim_signals::Error) -> Self {
        Self::Signal(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(v) => write!(f, "{v:#?}"),
            Self::Utf8(v) => write!(f, "{v:#?}"),
            Self::AI(v) => write!(f, "{v:#?}"),
            Self::State(v) => write!(f, "{v:#?}"),
            Self::Signal(v) => write!(f, "{v:#?}"),
            Self::Custom(v) => write!(f, "{v}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(v) => write!(f, "{v}"),
            Self::Utf8(v) => write!(f, "{v}"),
            Self::AI(v) => write!(f, "{v}"),
            Self::State(v) => write!(f, "{v}"),
            Self::Signal(v) => write!(f, "{v}"),
            Self::Custom(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(v) => Some(v),
            Self::Utf8(v) => Some(v),
            Self::AI(v) => Some(v),
            Self::State(v) => Some(v),
            Self::Signal(v) => Some(v),
            _ => None,
        }
    }
}
