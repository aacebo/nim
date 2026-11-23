use crate::chat::{Media, ToolCall};
use crate::types::Annotation;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum ChatMessage {
    System(ChatSystemMessage),
    User(ChatUserMessage),
    Assistant(ChatAssistantMessage),
    Tool(ChatToolMessage),
}

impl ChatMessage {
    pub fn role(&self) -> &str {
        match self {
            Self::System(_) => "system",
            Self::User(_) => "user",
            Self::Assistant(_) => "assistant",
            Self::Tool(_) => "tool",
        }
    }

    pub fn content(&self) -> Option<&ChatMessageContent> {
        match self {
            Self::System(v) => Some(&v.content),
            Self::User(v) => Some(&v.content),
            Self::Assistant(v) => v.content.as_ref(),
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

impl From<ChatAssistantMessage> for ChatMessage {
    fn from(value: ChatAssistantMessage) -> Self {
        Self::Assistant(value)
    }
}

impl From<ChatToolMessage> for ChatMessage {
    fn from(value: ChatToolMessage) -> Self {
        Self::Tool(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatSystemMessage {
    pub content: ChatMessageContent,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatUserMessage {
    pub content: ChatMessageContent,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatAssistantMessage {
    pub content: Option<ChatMessageContent>,
    pub reasoning: Option<String>,

    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,

    #[serde(default)]
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatToolMessage {
    pub tool_call_id: String,
    pub content: Option<ChatMessageContent>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatMessageContent {
    Text(String),
    Parts(Vec<ChatMessageContentPart>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatMessageContentPart {
    Text { text: String },
    Json { json: serde_json::Value },
    Url { url: url::Url },
    Image(Media),
    Audio(Media),
    Video(Media),
}
