use crate::chat::ChatMessageContent;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatMessageEvent {
    pub sequence: usize,
    #[serde(flatten)]
    pub content: ChatMessageEventContent,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatMessageEventContent {
    Append { content: ChatMessageContent },
    Complete,
    Error { error: String },
}
