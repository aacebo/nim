#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Visual,
    Audio,
    Memory,
    Emotion,
    Executive,
    Action,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Visual => write!(f, "visual"),
            Self::Audio => write!(f, "audio"),
            Self::Memory => write!(f, "memory"),
            Self::Emotion => write!(f, "emotion"),
            Self::Executive => write!(f, "executive"),
            Self::Action => write!(f, "action"),
        }
    }
}
