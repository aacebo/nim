pub use nim_store::TaskStatus;

use crate::{ArtifactId, ModelId, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub parent_id: Option<TaskId>,
    pub version: Version,
    pub name: String,
    pub source: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(uuid::Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for TaskId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<TaskId> for uuid::Uuid {
    fn from(value: TaskId) -> Self {
        value.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task_{}", self.0.simple())
    }
}

impl serde::Serialize for TaskId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TaskId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("task_")
            .ok_or_else(|| serde::de::Error::custom("invalid task id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl From<nim_store::TaskRow> for Task {
    fn from(value: nim_store::TaskRow) -> Self {
        Self {
            id: value.id.into(),
            parent_id: value.parent_id.map(Into::into),
            version: value.version.into(),
            name: value.name,
            source: value.source,
            status: value.status.into_inner(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelTask {
    pub model_id: ModelId,
    pub task_id: TaskId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::ModelTaskRow> for ModelTask {
    fn from(value: nim_store::ModelTaskRow) -> Self {
        Self {
            model_id: value.model_id.into(),
            task_id: value.task_id.into(),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactTask {
    pub artifact_id: ArtifactId,
    pub task_id: TaskId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::ArtifactTaskRow> for ArtifactTask {
    fn from(value: nim_store::ArtifactTaskRow) -> Self {
        Self {
            artifact_id: value.artifact_id.into(),
            task_id: value.task_id.into(),
            created_at: value.created_at,
        }
    }
}
