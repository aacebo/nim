use crate::{Embedding, MemoryId, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entity {
    pub id: EntityId,
    #[serde(rename = "type")]
    pub ty: EntityType,
    pub version: Version,
    pub name: String,
    pub summary: Option<String>,
    pub confidence: f32,
    #[serde(skip)]
    pub embedding: Option<Embedding>,
    pub recalls: u64,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(uuid::Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for EntityId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<EntityId> for uuid::Uuid {
    fn from(value: EntityId) -> Self {
        value.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "entity_{}", self.0.simple())
    }
}

impl std::str::FromStr for EntityId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("entity_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid entity id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
    }
}

impl serde::Serialize for EntityId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for EntityId {
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
pub enum EntityType {
    Person,
    Place,
    Organization,
    Project,
    Object,
    Event,
    Other,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Person => write!(f, "person"),
            Self::Place => write!(f, "place"),
            Self::Organization => write!(f, "organization"),
            Self::Project => write!(f, "project"),
            Self::Object => write!(f, "object"),
            Self::Event => write!(f, "event"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl TryFrom<nim_store::EntityRow> for Entity {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::EntityRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            ty: serde_json::from_value(serde_json::Value::String(value.ty))?,
            version: value.version.into(),
            name: value.name,
            summary: value.summary,
            confidence: value.confidence as f32,
            embedding: value.embedding.map(Into::into),
            recalls: value.recalls as u64,
            recalled_at: value.recalled_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityRelation {
    #[serde(rename = "type")]
    pub ty: EntityRelationType,
    pub source: EntityId,
    pub target: EntityId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityRelationType {
    PartOf,
    MemberOf,
    LocatedIn,
    WorksOn,
    Owns,
    RelatedTo,
}

impl std::fmt::Display for EntityRelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PartOf => write!(f, "part_of"),
            Self::MemberOf => write!(f, "member_of"),
            Self::LocatedIn => write!(f, "located_in"),
            Self::WorksOn => write!(f, "works_on"),
            Self::Owns => write!(f, "owns"),
            Self::RelatedTo => write!(f, "related_to"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityMemory {
    pub entity_id: EntityId,
    pub memory_id: MemoryId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::EntityMemoryRow> for EntityMemory {
    fn from(value: nim_store::EntityMemoryRow) -> Self {
        Self {
            entity_id: value.entity_id.into(),
            memory_id: value.memory_id.into(),
            created_at: value.created_at,
        }
    }
}

impl TryFrom<nim_store::EntityRelationRow> for EntityRelation {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::EntityRelationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            ty: serde_json::from_value(serde_json::Value::String(value.ty))?,
            source: value.source_id.into(),
            target: value.target_id.into(),
            created_at: value.created_at,
        })
    }
}
