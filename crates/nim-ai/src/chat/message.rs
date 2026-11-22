use hf_chat_template as hf;

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

    pub fn retain_text_parts(&mut self) {
        let content = match self {
            Self::System(message) => Some(&mut message.content),
            Self::User(message) => Some(&mut message.content),
            Self::Assistant(message) => message.content.as_mut(),
            Self::Tool(message) => message.content.as_mut(),
        };

        if let Some(ChatMessageContent::Parts(parts)) = content {
            parts.retain(|part| matches!(part, ChatMessageContentPart::Text { .. }));
        }
    }

    pub fn has_non_text_parts(&self) -> bool {
        self.content().is_some_and(|content| {
            matches!(content, ChatMessageContent::Parts(parts) if parts.iter().any(|part| !matches!(part, ChatMessageContentPart::Text { .. })))
        })
    }

    pub fn into_hf(self) -> Result<hf::Message, nim_error::Diagnostic> {
        match self {
            Self::System(v) => v.into_hf(),
            Self::User(v) => v.into_hf(),
            Self::Assistant(v) => v.into_hf(),
            Self::Tool(v) => v.into_hf(),
        }
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

impl ChatSystemMessage {
    pub fn into_hf(self) -> Result<hf::Message, nim_error::Diagnostic> {
        Ok(hf::Message {
            role: "system".into(),
            content: Some(self.content.into_hf()?),
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatUserMessage {
    pub content: ChatMessageContent,
}

impl ChatUserMessage {
    pub fn into_hf(self) -> Result<hf::Message, nim_error::Diagnostic> {
        Ok(hf::Message {
            role: "user".into(),
            content: Some(self.content.into_hf()?),
            ..Default::default()
        })
    }
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

impl ChatAssistantMessage {
    pub fn from_qwen(text: String, compat: super::CompatMode) -> nim_error::Result<Self> {
        let mut text = text.trim().to_string();
        let mut reasoning = None;

        if let Some(value) = text.strip_prefix("<think>") {
            match value.find("</think>") {
                Some(index) => {
                    reasoning = Some(value[..index].trim().to_string());
                    text = value[index + "</think>".len()..].trim().to_string();
                }
                None if compat == super::CompatMode::Error => {
                    return Err(nim_error::Diagnostic::message("unterminated Qwen reasoning block"));
                }
                None => {}
            }
        }

        let mut content = String::new();
        let mut tool_calls = Vec::new();
        let mut rest = text.as_str();

        while let Some(start) = rest.find("<tool_call>") {
            content.push_str(&rest[..start]);

            let value = &rest[start + "<tool_call>".len()..];
            let Some(end) = value.find("</tool_call>") else {
                if compat == super::CompatMode::Error {
                    return Err(nim_error::Diagnostic::message("unterminated Qwen tool call"));
                }

                content.push_str(&rest[start..]);
                rest = "";
                break;
            };

            let raw = value[..end].trim();
            let parsed = (|| -> nim_error::Result<ToolCall> {
                let value: serde_json::Value = serde_json::from_str(raw)?;
                let object = value
                    .as_object()
                    .ok_or_else(|| nim_error::Diagnostic::message("Qwen tool call must be an object"))?;
                let name = object
                    .get("name")
                    .or_else(|| object.get("function").and_then(|v| v.get("name")))
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| nim_error::Diagnostic::message("Qwen tool call is missing a name"))?
                    .to_string();
                let args = object
                    .get("arguments")
                    .or_else(|| object.get("args"))
                    .or_else(|| object.get("function").and_then(|v| v.get("arguments")))
                    .map(|value| match value {
                        serde_json::Value::String(value) => serde_json::from_str(value),
                        serde_json::Value::Object(value) => Ok(value.clone()),
                        _ => Err(serde_json::Error::io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Qwen tool arguments must be an object",
                        ))),
                    })
                    .transpose()?
                    .unwrap_or_default();
                let id = object
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("call_{}", tool_calls.len()));

                Ok(ToolCall { id, name, args })
            })();

            match parsed {
                Ok(call) => tool_calls.push(call),
                Err(error) if compat == super::CompatMode::Error => return Err(error),
                Err(_) => content.push_str(&rest[start..start + "<tool_call>".len() + end + "</tool_call>".len()]),
            }

            rest = &value[end + "</tool_call>".len()..];
        }

        content.push_str(rest);
        let content = (!content.trim().is_empty()).then(|| ChatMessageContent::Text(content.trim().to_string()));

        Ok(Self {
            content,
            reasoning,
            tool_calls,
            annotations: Vec::new(),
        })
    }

    pub fn into_hf(self) -> Result<hf::Message, nim_error::Diagnostic> {
        let results: Vec<_> = self.tool_calls.iter().map(serde_json::to_value).collect();
        let (tool_calls, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        let tool_calls: Vec<_> = tool_calls.into_iter().map(Result::unwrap).collect();
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

        if !errors.is_empty() {
            return Err(nim_error::Diagnostic::new().append(errors));
        }

        let mut extra = serde_json::Map::new();

        if let Some(v) = &self.reasoning {
            extra.extend([
                ("reasoning".into(), serde_json::to_value(v)?),
                ("reasoning_content".into(), serde_json::to_value(v)?),
                ("thinking".into(), serde_json::to_value(v)?),
            ]);
        }

        Ok(hf::Message {
            role: "assistant".into(),
            content: match self.content {
                None => None,
                Some(v) => Some(v.into_hf()?),
            },
            tool_calls,
            extra,
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatToolMessage {
    pub tool_call_id: String,
    pub content: Option<ChatMessageContent>,
}

impl ChatToolMessage {
    pub fn into_hf(self) -> Result<hf::Message, nim_error::Diagnostic> {
        Ok(hf::Message {
            role: "tool".into(),
            content: match self.content {
                None => None,
                Some(v) => Some(v.into_hf()?),
            },
            extra: [("tool_call_id".into(), serde_json::to_value(&self.tool_call_id)?)]
                .into_iter()
                .collect(),
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatMessageContent {
    Text(String),
    Parts(Vec<ChatMessageContentPart>),
}

impl ChatMessageContent {
    pub fn into_hf(self) -> Result<hf::Content, nim_error::Diagnostic> {
        match self {
            Self::Text(text) => Ok(hf::Content::Text(text)),
            Self::Parts(parts) => {
                let mut out = Vec::new();

                for part in &parts {
                    out.push(serde_json::to_value(part)?);
                }

                Ok(hf::Content::Parts(out))
            }
        }
    }

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
