pub use nim_store::{LocalModelSource, ModelResource, ModelSource};

use crate::Version;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub id: ModelId,
    pub version: Version,
    #[serde(rename = "type")]
    pub ty: String,
    pub name: String,
    pub tasks: Vec<String>,
    pub architectures: Vec<String>,
    pub source: ModelSource,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelId(uuid::Uuid);

impl ModelId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for ModelId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<ModelId> for uuid::Uuid {
    fn from(value: ModelId) -> Self {
        value.0
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "model_{}", self.0.simple())
    }
}

impl serde::Serialize for ModelId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ModelId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("model_")
            .ok_or_else(|| serde::de::Error::custom("invalid model id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl From<nim_store::ModelRow> for Model {
    fn from(value: nim_store::ModelRow) -> Self {
        Self {
            id: value.id.into(),
            version: value.version.into(),
            ty: value.ty,
            name: value.name,
            tasks: value.tasks,
            architectures: value.architectures,
            source: value.source.into_inner(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelWeight {
    pub id: ModelWeightId,
    pub model_id: ModelId,
    pub version: Version,
    pub name: String,
    pub content: ModelWeightContent,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<nim_store::ModelWeightRow> for ModelWeight {
    type Error = crate::Error;

    fn try_from(value: nim_store::ModelWeightRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            model_id: value.model_id.into(),
            version: value.version.into(),
            name: value.name,
            content: serde_json::from_value(value.content)?,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelWeightId(uuid::Uuid);

impl ModelWeightId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for ModelWeightId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<ModelWeightId> for uuid::Uuid {
    fn from(value: ModelWeightId) -> Self {
        value.0
    }
}

impl std::fmt::Display for ModelWeightId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "model_weight_{}", self.0.simple())
    }
}

impl serde::Serialize for ModelWeightId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ModelWeightId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("model_weight_")
            .ok_or_else(|| serde::de::Error::custom("invalid model weight id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelWeightContent {
    Platt { a: f32, b: f32 },
}
