use crate::chat::{Media, ToolCall};
use crate::types::Annotation;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role")]
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

    pub fn retain_supported_parts(&mut self, vision: bool, audio: bool) {
        let content = match self {
            Self::System(message) => Some(&mut message.content),
            Self::User(message) => Some(&mut message.content),
            Self::Assistant(message) => message.content.as_mut(),
            Self::Tool(message) => message.content.as_mut(),
        };

        if let Some(ChatMessageContent::Parts(parts)) = content {
            parts.retain(|part| match part {
                ChatMessageContentPart::Text { .. } => true,
                ChatMessageContentPart::Image(_) | ChatMessageContentPart::Video(_) => vision,
                ChatMessageContentPart::Audio(_) => audio,
            });
        }
    }

    pub fn has_unsupported_parts(&self, vision: bool, audio: bool) -> bool {
        self.content().is_some_and(|content| {
            matches!(content, ChatMessageContent::Parts(parts) if parts.iter().any(|part| match part {
                ChatMessageContentPart::Text { .. } => false,
                ChatMessageContentPart::Image(_) | ChatMessageContentPart::Video(_) => !vision,
                ChatMessageContentPart::Audio(_) => !audio,
            }))
        })
    }

    pub fn into_openai(self, compat: super::CompatMode) -> nim_error::Result<Option<serde_json::Value>> {
        let content = |content: ChatMessageContent| content.into_openai(compat);

        match self {
            Self::System(message) => {
                Ok(content(message.content)?.map(|content| serde_json::json!({ "role": "system", "content": content })))
            }
            Self::User(message) => {
                Ok(content(message.content)?.map(|content| serde_json::json!({ "role": "user", "content": content })))
            }
            Self::Assistant(message) => Ok(Some(serde_json::json!({
                "role": "assistant",
                "content": message.content.map(content).transpose()?,
                "tool_calls": message.tool_calls.into_iter().map(ToolCall::into_openai).collect::<Vec<_>>(),
            }))),
            Self::Tool(message) => Ok(Some(serde_json::json!({
                "role": "tool",
                "tool_call_id": message.tool_call_id,
                "content": message.content.map(content).transpose()?,
            }))),
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

impl ChatMessageContent {
    pub fn into_openai(self, compat: super::CompatMode) -> nim_error::Result<Option<serde_json::Value>> {
        match self {
            Self::Text(text) => Ok(Some(serde_json::Value::String(text))),
            Self::Parts(parts) => {
                let mut output = Vec::new();

                for part in parts {
                    match part {
                        ChatMessageContentPart::Text { text } => output.push(serde_json::json!({ "type": "text", "text": text })),
                        ChatMessageContentPart::Image(Media::Url { url, summary }) => {
                            output.push(serde_json::json!({
                                "type": "image",
                                "image": url,
                                "summary": summary,
                            }));
                        }
                        ChatMessageContentPart::Image(Media::Path { path, summary }) => {
                            output.push(serde_json::json!({
                                "type": "image",
                                "image": path,
                                "summary": summary,
                            }));
                        }
                        ChatMessageContentPart::Audio(Media::Url { url, summary }) => {
                            output.push(serde_json::json!({
                                "type": "audio",
                                "image": url ,
                                "summary": summary,
                            }));
                        }
                        ChatMessageContentPart::Audio(Media::Path { path, summary }) => {
                            output.push(serde_json::json!({
                                "type": "audio",
                                "image": path,
                                "summary": summary,
                            }));
                        }
                        part if compat == super::CompatMode::Ignore => {
                            let _ = part;
                        }
                        _ => {
                            return Err(nim_error::Diagnostic::message(
                                "remote completion does not support this media content",
                            ));
                        }
                    }
                }

                if output.is_empty() && compat == super::CompatMode::Ignore {
                    Ok(None)
                } else {
                    (!output.is_empty())
                        .then_some(serde_json::Value::Array(output))
                        .ok_or_else(|| nim_error::Diagnostic::message("message has no supported content"))
                        .map(Some)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatMessageContentPart {
    Text { text: String },
    Image(Media),
    Audio(Media),
    Video(Media),
}
