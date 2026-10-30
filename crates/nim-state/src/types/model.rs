pub use nim_store::{LocalModelSource, ModelSource, ModelWeights};

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
    pub weights: Vec<ModelWeights>,
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
            weights: value.weights.into_inner(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
