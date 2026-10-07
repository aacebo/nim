use crate::{EntityId, FactId, MemoryId, PredictionId, Region};

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
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
