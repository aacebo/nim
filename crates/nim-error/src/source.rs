use std::sync::Arc;

#[derive(Clone)]
pub enum Source {
    None,
    Message(String),
    Error(Arc<dyn Send + Sync + std::error::Error>),
}

impl Source {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "null"),
            Self::Message(v) => write!(f, "{v}"),
            Self::Error(err) => write!(f, "{err:#?}"),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "null"),
            Self::Message(v) => write!(f, "{v}"),
            Self::Error(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Source {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Error(v) => Some(v),
            _ => None,
        }
    }
}
