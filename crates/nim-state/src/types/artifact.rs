pub use nim_store::ArtifactContent;

use crate::{AnnotationPartial, Embedding, MemoryId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub name: String,
    #[serde(flatten)]
    pub content: ArtifactContent,
    pub summary: Option<String>,
    pub embedding: Option<Embedding>,
    pub annotations: Vec<AnnotationPartial>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::ArtifactRow> for Artifact {
    fn from(value: nim_store::ArtifactRow) -> Self {
        Self {
            id: value.id.into(),
            name: value.name,
            content: value.content.into_inner(),
            summary: value.summary,
            embedding: value.embedding.map(Into::into),
            annotations: Default::default(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryArtifact {
    pub memory_id: MemoryId,
    pub artifact_id: ArtifactId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::MemoryArtifactRow> for MemoryArtifact {
    fn from(value: nim_store::MemoryArtifactRow) -> Self {
        Self {
            memory_id: value.memory_id.into(),
            artifact_id: value.artifact_id.into(),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArtifactId(uuid::Uuid);

impl ArtifactId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for ArtifactId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<ArtifactId> for uuid::Uuid {
    fn from(value: ArtifactId) -> Self {
        value.0
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "artifact_{}", self.0.simple())
    }
}

impl serde::Serialize for ArtifactId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ArtifactId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("artifact_")
            .ok_or_else(|| serde::de::Error::custom("invalid artifact id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}
