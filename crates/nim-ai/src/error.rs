pub enum Error {
    Candle(candle_core::Error),
    IO(std::io::Error),
    Http(reqwest::Error),
    Hf(hf_hub::api::sync::ApiError),
    Json(serde_json::Error),
    Parse(ParseError),
    Custom { group: String, message: String },
}

impl Error {
    pub fn custom(group: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self::Custom {
            group: group.to_string(),
            message: message.to_string(),
        }
    }
}

impl From<candle_core::Error> for Error {
    fn from(value: candle_core::Error) -> Self {
        Self::Candle(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<hf_hub::api::sync::ApiError> for Error {
    fn from(value: hf_hub::api::sync::ApiError) -> Self {
        Self::Hf(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Custom {
            group: "unknown".to_string(),
            message: value.to_string(),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Candle(v) => write!(f, "{v:#?}"),
            Self::IO(v) => write!(f, "{v:#?}"),
            Self::Http(v) => write!(f, "{v:#?}"),
            Self::Hf(v) => write!(f, "{v:#?}"),
            Self::Json(v) => write!(f, "{v:#?}"),
            Self::Parse(v) => write!(f, "{v:#?}"),
            Self::Custom { group, message } => write!(f, "[error::{group}] -> {message}"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Candle(v) => write!(f, "{v}"),
            Self::IO(v) => write!(f, "{v}"),
            Self::Http(v) => write!(f, "{v}"),
            Self::Hf(v) => write!(f, "{v}"),
            Self::Json(v) => write!(f, "{v}"),
            Self::Parse(v) => write!(f, "{v}"),
            Self::Custom { group, message } => write!(f, "[error::{group}] -> {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Candle(v) => Some(v),
            Self::IO(v) => Some(v),
            Self::Http(v) => Some(v),
            Self::Hf(v) => Some(v),
            Self::Json(v) => Some(v),
            Self::Parse(v) => Some(v),
            _ => None,
        }
    }
}

pub enum ParseError {
    Int(std::num::ParseIntError),
    Float(std::num::ParseFloatError),
    Url(url::ParseError),
}

impl From<std::num::ParseIntError> for ParseError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::Int(value)
    }
}

impl From<std::num::ParseFloatError> for ParseError {
    fn from(value: std::num::ParseFloatError) -> Self {
        Self::Float(value)
    }
}

impl From<url::ParseError> for ParseError {
    fn from(value: url::ParseError) -> Self {
        Self::Url(value)
    }
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v:#?}"),
            Self::Float(v) => write!(f, "{v:#?}"),
            Self::Url(v) => write!(f, "{v:#?}"),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Url(v) => write!(f, "{v}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Int(v) => Some(v),
            Self::Float(v) => Some(v),
            Self::Url(v) => Some(v),
        }
    }
}
