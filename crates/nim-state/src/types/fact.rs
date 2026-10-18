use crate::Embedding;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub description: String,
    pub confidence: f32,
    pub embedding: Option<Embedding>,
    pub recalls: u64,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactId(uuid::Uuid);

impl FactId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for FactId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<FactId> for uuid::Uuid {
    fn from(value: FactId) -> Self {
        value.0
    }
}

impl std::fmt::Display for FactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fact_{}", self.0.simple())
    }
}

impl serde::Serialize for FactId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for FactId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("fact_")
            .ok_or_else(|| serde::de::Error::custom("invalid fact id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl From<nim_store::FactRow> for Fact {
    fn from(value: nim_store::FactRow) -> Self {
        Self {
            id: value.id.into(),
            description: value.description,
            confidence: value.confidence,
            embedding: value.embedding.map(Into::into),
            recalls: value.recalls as u64,
            recalled_at: value.recalled_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
