use crate::{Embedding, Version};

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
