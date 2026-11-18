use crate::types::{Annotation, FileFormat, Status};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role")]
pub enum ChatMessage {
    System(ChatSystemMessage),
    User(ChatUserMessage),
    Assistant(ChatAssistantMessage),
}

impl From<ChatSystemMessage> for ChatMessage {
    fn from(value: ChatSystemMessage) -> Self {
        Self::System(value)
    }
}

impl From<ChatUserMessage> for ChatMessage {
    fn from(value: ChatUserMessage) -> Self {
        Self::User(value)
    }
}

impl From<ChatAssistantMessage> for ChatMessage {
    fn from(value: ChatAssistantMessage) -> Self {
        Self::Assistant(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatSystemMessage {
    pub content: Vec<ChatMessageContent>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatUserMessage {
    pub content: Vec<ChatMessageContent>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatAssistantMessage {
    pub status: Status,
    pub content: Vec<ChatMessageContent>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatMessageContent {
    Text {
        text: String,
    },
    Url {
        url: url::Url,
    },
    /// base64 encoded file data
    File {
        name: Option<String>,
        data: String,
        format: FileFormat,
    },
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
    },
    ToolOutput {
        call_id: String,
        content: Vec<Self>,
    },
    Reasoning {
        text: String,
    },
}
