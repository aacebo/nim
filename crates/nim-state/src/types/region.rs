#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Vision,
    Audio,
    Memory,
    Emotion,
    Executive,
    Action,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vision => write!(f, "vision"),
            Self::Audio => write!(f, "audio"),
            Self::Memory => write!(f, "memory"),
            Self::Emotion => write!(f, "emotion"),
            Self::Executive => write!(f, "executive"),
            Self::Action => write!(f, "action"),
        }
    }
}

impl From<nim_store::Region> for Region {
    fn from(value: nim_store::Region) -> Self {
        match value {
            nim_store::Region::Vision => Self::Vision,
            nim_store::Region::Audio => Self::Audio,
            nim_store::Region::Memory => Self::Memory,
            nim_store::Region::Emotion => Self::Emotion,
            nim_store::Region::Executive => Self::Executive,
            nim_store::Region::Action => Self::Action,
            _ => unreachable!(),
        }
    }
}

impl From<Region> for nim_store::Region {
    fn from(value: Region) -> Self {
        match value {
            Region::Vision => Self::Vision,
            Region::Audio => Self::Audio,
            Region::Memory => Self::Memory,
            Region::Emotion => Self::Emotion,
            Region::Executive => Self::Executive,
            Region::Action => Self::Action,
        }
    }
}
