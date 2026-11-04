use nim_state::{ArtifactId, ChatId, MessageId, TaskId};

#[derive(Debug, Default, Clone)]
pub struct TaskOptions {
    pub parent_id: Option<TaskId>,
    pub sources: Vec<TaskSource>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskSource {
    Artifact(ArtifactId),
    Chat(ChatId),
    Message(MessageId),
}

impl From<ArtifactId> for TaskSource {
    fn from(value: ArtifactId) -> Self {
        Self::Artifact(value)
    }
}

impl From<ChatId> for TaskSource {
    fn from(value: ChatId) -> Self {
        Self::Chat(value)
    }
}

impl From<MessageId> for TaskSource {
    fn from(value: MessageId) -> Self {
        Self::Message(value)
    }
}
