use std::collections::BTreeMap;

use crate::{Annotation, ChatMessageContent, ToolCall};

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageRole {
    System,
    Agent,
    User,
    Tool,
}

impl std::fmt::Display for ChatMessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Agent => write!(f, "agent"),
            Self::User => write!(f, "user"),
            Self::Tool => write!(f, "tool"),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum ChatMessage {
    System(ChatSystemMessage),
    User(ChatUserMessage),
    Agent(ChatAgentMessage),
    Tool(ChatToolMessage),
}

impl ChatMessage {
    pub fn role(&self) -> ChatMessageRole {
        match self {
            Self::System(_) => ChatMessageRole::System,
            Self::User(_) => ChatMessageRole::User,
            Self::Agent(_) => ChatMessageRole::Agent,
            Self::Tool(_) => ChatMessageRole::Tool,
        }
    }

    pub fn content(&self) -> Option<&ChatMessageContent> {
        match self {
            Self::System(v) => Some(&v.content),
            Self::User(v) => Some(&v.content),
            Self::Agent(v) => v.content.as_ref(),
            Self::Tool(v) => v.content.as_ref(),
        }
    }
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

impl From<ChatAgentMessage> for ChatMessage {
    fn from(value: ChatAgentMessage) -> Self {
        Self::Agent(value)
    }
}

impl From<ChatToolMessage> for ChatMessage {
    fn from(value: ChatToolMessage) -> Self {
        Self::Tool(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatSystemMessage {
    pub name: Option<String>,
    pub content: ChatMessageContent,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatUserMessage {
    pub name: Option<String>,
    pub content: ChatMessageContent,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatAgentMessage {
    pub name: Option<String>,
    pub content: Option<ChatMessageContent>,

    pub reasoning: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<Annotation>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatToolMessage {
    pub tool_call_id: String,
    pub content: Option<ChatMessageContent>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
