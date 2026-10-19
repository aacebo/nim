use crate::{EntityId, FactId, MemoryId, PredictionId, Region};

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Source {
    System,
    Region(Region),
    Memory(MemoryId),
    Entity(EntityId),
    Prediction(PredictionId),
    Fact(FactId),
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Region(v) => write!(f, "region::{v}"),
            Self::Memory(v) => write!(f, "{v}"),
            Self::Entity(v) => write!(f, "{v}"),
            Self::Prediction(v) => write!(f, "{v}"),
            Self::Fact(v) => write!(f, "{v}"),
        }
    }
}

impl From<nim_store::Source> for Source {
    fn from(value: nim_store::Source) -> Self {
        match value {
            nim_store::Source::System => Self::System,
            nim_store::Source::Region(v) => Self::Region(v),
            nim_store::Source::Memory(v) => Self::Memory(v.into()),
            nim_store::Source::Entity(v) => Self::Entity(v.into()),
            nim_store::Source::Prediction(v) => Self::Prediction(v.into()),
            nim_store::Source::Fact(v) => Self::Fact(v.into()),
            _ => unreachable!(),
        }
    }
}

impl From<Source> for nim_store::Source {
    fn from(value: Source) -> Self {
        match value {
            Source::System => Self::System,
            Source::Region(v) => Self::Region(v),
            Source::Memory(v) => Self::Memory(v.into()),
            Source::Entity(v) => Self::Entity(v.into()),
            Source::Prediction(v) => Self::Prediction(v.into()),
            Source::Fact(v) => Self::Fact(v.into()),
        }
    }
}
