use crate::MemoryId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Recall {
    pub id: RecallId,
    pub trigger: RecallTrigger,
    pub source: RecallSource,
    pub query: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecallId(uuid::Uuid);

impl RecallId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for RecallId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<RecallId> for uuid::Uuid {
    fn from(value: RecallId) -> Self {
        value.0
    }
}

impl std::fmt::Display for RecallId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "recall_{}", self.0.simple())
    }
}

impl serde::Serialize for RecallId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for RecallId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("recall_")
            .ok_or_else(|| serde::de::Error::custom("invalid recall id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallTrigger {
    Explicit,
    Contextual,
    Associative,
    Sensory,
    Temporal,
}

impl std::fmt::Display for RecallTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit => write!(f, "explicit"),
            Self::Contextual => write!(f, "contextual"),
            Self::Associative => write!(f, "associative"),
            Self::Sensory => write!(f, "sensory"),
            Self::Temporal => write!(f, "temporal"),
        }
    }
}

impl From<nim_store::RecallTrigger> for RecallTrigger {
    fn from(value: nim_store::RecallTrigger) -> Self {
        match value {
            nim_store::RecallTrigger::Explicit => Self::Explicit,
            nim_store::RecallTrigger::Contextual => Self::Contextual,
            nim_store::RecallTrigger::Associative => Self::Associative,
            nim_store::RecallTrigger::Sensory => Self::Sensory,
            nim_store::RecallTrigger::Temporal => Self::Temporal,
            _ => unreachable!(),
        }
    }
}

impl From<RecallTrigger> for nim_store::RecallTrigger {
    fn from(value: RecallTrigger) -> Self {
        match value {
            RecallTrigger::Explicit => Self::Explicit,
            RecallTrigger::Contextual => Self::Contextual,
            RecallTrigger::Associative => Self::Associative,
            RecallTrigger::Sensory => Self::Sensory,
            RecallTrigger::Temporal => Self::Temporal,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallSource {
    User,
    Agent,
    System,
}

impl std::fmt::Display for RecallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Agent => write!(f, "agent"),
            Self::System => write!(f, "system"),
        }
    }
}

impl From<nim_store::RecallSource> for RecallSource {
    fn from(value: nim_store::RecallSource) -> Self {
        match value {
            nim_store::RecallSource::User => Self::User,
            nim_store::RecallSource::Agent => Self::Agent,
            nim_store::RecallSource::System => Self::System,
            _ => unreachable!(),
        }
    }
}

impl From<RecallSource> for nim_store::RecallSource {
    fn from(value: RecallSource) -> Self {
        match value {
            RecallSource::User => Self::User,
            RecallSource::Agent => Self::Agent,
            RecallSource::System => Self::System,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRecall {
    pub memory_id: MemoryId,
    pub recall_id: RecallId,
    pub score: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::RecallRow> for Recall {
    fn from(value: nim_store::RecallRow) -> Self {
        Self {
            id: value.id.into(),
            trigger: value.trigger.into(),
            source: value.source.into(),
            query: value.query,
            created_at: value.created_at,
        }
    }
}

impl From<nim_store::MemoryRecallRow> for MemoryRecall {
    fn from(value: nim_store::MemoryRecallRow) -> Self {
        Self {
            memory_id: value.memory_id.into(),
            recall_id: value.recall_id.into(),
            score: value.score,
            created_at: value.created_at,
        }
    }
}
