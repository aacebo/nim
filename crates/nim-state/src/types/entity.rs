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
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("entity_")
            .ok_or_else(|| serde::de::Error::custom("invalid entity id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
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

impl From<nim_store::EntityRow> for Entity {
    fn from(value: nim_store::EntityRow) -> Self {
        Self {
            id: value.id.into(),
            ty: value.ty.into(),
            version: value.version.into(),
            name: value.name,
            summary: value.summary,
            confidence: value.confidence as f32,
            embedding: value.embedding.map(Into::into),
            recalls: value.recalls as u64,
            recalled_at: value.recalled_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<nim_store::EntityType> for EntityType {
    fn from(value: nim_store::EntityType) -> Self {
        match value {
            nim_store::EntityType::Person => Self::Person,
            nim_store::EntityType::Place => Self::Place,
            nim_store::EntityType::Organization => Self::Organization,
            nim_store::EntityType::Project => Self::Project,
            nim_store::EntityType::Object => Self::Object,
            nim_store::EntityType::Event => Self::Event,
            nim_store::EntityType::Other => Self::Other,
            _ => unreachable!(),
        }
    }
}

impl From<EntityType> for nim_store::EntityType {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Person => Self::Person,
            EntityType::Place => Self::Place,
            EntityType::Organization => Self::Organization,
            EntityType::Project => Self::Project,
            EntityType::Object => Self::Object,
            EntityType::Event => Self::Event,
            EntityType::Other => Self::Other,
        }
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

impl From<nim_store::EntityRelationRow> for EntityRelation {
    fn from(value: nim_store::EntityRelationRow) -> Self {
        Self {
            ty: value.ty.into(),
            source: value.source_id.into(),
            target: value.target_id.into(),
            created_at: value.created_at,
        }
    }
}

impl From<nim_store::EntityRelationType> for EntityRelationType {
    fn from(value: nim_store::EntityRelationType) -> Self {
        match value {
            nim_store::EntityRelationType::PartOf => Self::PartOf,
            nim_store::EntityRelationType::MemberOf => Self::MemberOf,
            nim_store::EntityRelationType::LocatedIn => Self::LocatedIn,
            nim_store::EntityRelationType::WorksOn => Self::WorksOn,
            nim_store::EntityRelationType::Owns => Self::Owns,
            nim_store::EntityRelationType::RelatedTo => Self::RelatedTo,
        }
    }
}

impl From<EntityRelationType> for nim_store::EntityRelationType {
    fn from(value: EntityRelationType) -> Self {
        match value {
            EntityRelationType::PartOf => Self::PartOf,
            EntityRelationType::MemberOf => Self::MemberOf,
            EntityRelationType::LocatedIn => Self::LocatedIn,
            EntityRelationType::WorksOn => Self::WorksOn,
            EntityRelationType::Owns => Self::Owns,
            EntityRelationType::RelatedTo => Self::RelatedTo,
        }
    }
}
