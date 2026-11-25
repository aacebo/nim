use crate::Version;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ModelSource {
    Local(LocalModelSource),
    Remote {
        provider: String,
        id: String,
        base_url: Option<String>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalModelSource {
    Hub {
        id: String,
        #[serde(default)]
        resource: Option<ModelResource>,
    },
    Path {
        uri: String,
        format: String,
        path: Option<std::path::PathBuf>,
        size: u64,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelResource {
    pub uri: String,
    pub format: String,
    pub path: Option<std::path::PathBuf>,
    pub size: u64,
}

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

impl std::str::FromStr for ModelId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("model_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid model id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
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
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<nim_store::ModelRow> for Model {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::ModelRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            version: value.version.into(),
            ty: value.ty,
            name: value.name,
            tasks: value.tasks,
            architectures: value.architectures,
            source: serde_json::from_value(value.source)?,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
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
    type Error = nim_error::Diagnostic;

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

impl std::str::FromStr for ModelWeightId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("model_weight_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid model weight id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
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
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelWeightContent {
    Platt { a: f32, b: f32 },
}
