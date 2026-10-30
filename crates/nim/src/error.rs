pub enum Error {
    #[cfg(feature = "ai")]
    AI(nim_ai::Error),
    #[cfg(feature = "signals")]
    Signals(nim_signals::Error),
    State(nim_state::Error),
    #[cfg(feature = "runtime")]
    Runtime(nim_runtime::Error),
}

#[cfg(feature = "ai")]
impl From<nim_ai::Error> for Error {
    fn from(value: nim_ai::Error) -> Self {
        Self::AI(value)
    }
}

#[cfg(feature = "signals")]
impl From<nim_signals::Error> for Error {
    fn from(value: nim_signals::Error) -> Self {
        Self::Signals(value)
    }
}

#[cfg(feature = "runtime")]
impl From<nim_runtime::Error> for Error {
    fn from(value: nim_runtime::Error) -> Self {
        Self::Runtime(value)
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
            #[cfg(feature = "ai")]
            Self::AI(v) => write!(f, "{v:#?}"),
            #[cfg(feature = "signals")]
            Self::Signals(v) => write!(f, "{v:#?}"),
            #[cfg(feature = "runtime")]
            Self::Runtime(v) => write!(f, "{v:#?}"),
            Self::State(v) => write!(f, "{v:#?}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "ai")]
            Self::AI(v) => write!(f, "{v}"),
            #[cfg(feature = "signals")]
            Self::Signals(v) => write!(f, "{v}"),
            #[cfg(feature = "runtime")]
            Self::Runtime(v) => write!(f, "{v}"),
            Self::State(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "ai")]
            Self::AI(v) => Some(v),
            #[cfg(feature = "signals")]
            Self::Signals(v) => Some(v),
            #[cfg(feature = "runtime")]
            Self::Runtime(v) => Some(v),
            Self::State(v) => Some(v),
        }
    }
}
