use nim_store::Span;

use crate::{ArtifactId, ChatId, MessageId, ModelId, Source, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub parent_id: Option<TaskId>,
    pub version: Version,
    pub name: String,
    pub source: Source,
    #[serde(flatten)]
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

impl TryFrom<nim_store::TaskRow> for Task {
    type Error = crate::Error;

    fn try_from(value: nim_store::TaskRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            parent_id: value.parent_id.map(Into::into),
            version: value.version.into(),
            name: value.name,
            source: value.source.into_inner().into(),
            status: serde_json::from_value(value.status)?,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatTask {
    pub chat_id: ChatId,
    pub task_id: TaskId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::ChatTaskRow> for ChatTask {
    fn from(value: nim_store::ChatTaskRow) -> Self {
        Self {
            chat_id: value.chat_id.into(),
            task_id: value.task_id.into(),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageTask {
    pub message_id: MessageId,
    pub task_id: TaskId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::MessageTaskRow> for MessageTask {
    fn from(value: nim_store::MessageTaskRow) -> Self {
        Self {
            message_id: value.message_id.into(),
            task_id: value.task_id.into(),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Started {
        output: Option<TaskOutput>,
        started_at: chrono::DateTime<chrono::Utc>,
    },
    Complete {
        output: Option<TaskOutput>,
        started_at: chrono::DateTime<chrono::Utc>,
        ended_at: chrono::DateTime<chrono::Utc>,
    },
    Error {
        reason: String,
        started_at: chrono::DateTime<chrono::Utc>,
        ended_at: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TaskOutput {
    Object(TaskContent),
    Array(Vec<TaskContent>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskContent {
    Text { text: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskEvent {
    pub id: TaskEventId,
    pub task_id: TaskId,
    pub content: TaskEventContent,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskEventId(uuid::Uuid);

impl TaskEventId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for TaskEventId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<TaskEventId> for uuid::Uuid {
    fn from(value: TaskEventId) -> Self {
        value.0
    }
}

impl std::fmt::Display for TaskEventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task_event_{}", self.0.simple())
    }
}

impl serde::Serialize for TaskEventId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TaskEventId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("task_event_")
            .ok_or_else(|| serde::de::Error::custom("invalid task event id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl TryFrom<nim_store::TaskEventRow> for TaskEvent {
    type Error = crate::Error;

    fn try_from(value: nim_store::TaskEventRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            task_id: value.task_id.into(),
            content: serde_json::from_value(value.content)?,
            created_at: value.created_at,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskEventContent {
    TextDelta {
        span: Span,
        text: String,
        output_index: usize,
        content_index: usize,
    },
}
