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

impl std::str::FromStr for RecallId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("recall_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid recall id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
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
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRecall {
    pub memory_id: MemoryId,
    pub recall_id: RecallId,
    pub score: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<nim_store::RecallRow> for Recall {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::RecallRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            trigger: serde_json::from_value(serde_json::Value::String(value.trigger))?,
            source: serde_json::from_value(serde_json::Value::String(value.source))?,
            query: value.query,
            created_at: value.created_at,
        })
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
