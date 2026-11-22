use crate::{
    Chat, ChatAssistantMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatSystemMessage, ChatToolMessage,
    ChatUserMessage, CompatMode, Media, Render, Tool, ToolCall,
};

pub struct OpenAITemplate {
    compat: CompatMode,
}

impl OpenAITemplate {
    pub fn new(compat: CompatMode) -> Self {
        Self { compat }
    }
}

impl Render<Chat> for OpenAITemplate {
    type Output = Vec<serde_json::Value>;

    fn render(&self, value: &Chat) -> nim_error::Result<Self::Output> {
        value
            .messages
            .iter()
            .map(|message| self.render(message))
            .collect::<nim_error::Result<Vec<_>>>()
            .map(|messages| messages.into_iter().flatten().collect())
    }
}

impl Render<ChatMessage> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatMessage) -> nim_error::Result<Self::Output> {
        match value {
            ChatMessage::System(value) => self.render(value),
            ChatMessage::User(value) => self.render(value),
            ChatMessage::Assistant(value) => self.render(value),
            ChatMessage::Tool(value) => self.render(value),
        }
    }
}

impl Render<ChatSystemMessage> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatSystemMessage) -> nim_error::Result<Self::Output> {
        Ok(self
            .render(&value.content)?
            .map(|content| serde_json::json!({ "role": "system", "content": content })))
    }
}

impl Render<ChatUserMessage> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatUserMessage) -> nim_error::Result<Self::Output> {
        Ok(self
            .render(&value.content)?
            .map(|content| serde_json::json!({ "role": "user", "content": content })))
    }
}

impl Render<ChatAssistantMessage> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatAssistantMessage) -> nim_error::Result<Self::Output> {
        Ok(Some(serde_json::json!({
            "role": "assistant",
            "content": value.content.as_ref().map(|content| self.render(content)).transpose()?,
            "tool_calls": value.tool_calls.iter().map(|call| self.render(call)).collect::<nim_error::Result<Vec<_>>>()?,
        })))
    }
}

impl Render<ChatToolMessage> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatToolMessage) -> nim_error::Result<Self::Output> {
        Ok(Some(serde_json::json!({
            "role": "tool",
            "tool_call_id": value.tool_call_id,
            "content": value.content.as_ref().map(|content| self.render(content)).transpose()?,
        })))
    }
}

impl Render<ChatMessageContent> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatMessageContent) -> nim_error::Result<Self::Output> {
        match value {
            ChatMessageContent::Text(text) => Ok(Some(serde_json::Value::String(text.clone()))),
            ChatMessageContent::Parts(parts) => {
                let parts = parts
                    .iter()
                    .map(|part| self.render(part))
                    .collect::<nim_error::Result<Vec<_>>>()?
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                if parts.is_empty() && self.compat == CompatMode::Ignore {
                    Ok(None)
                } else if parts.is_empty() {
                    Err(nim_error::Diagnostic::message("message has no supported content"))
                } else {
                    Ok(Some(serde_json::Value::Array(parts)))
                }
            }
        }
    }
}

impl Render<ChatMessageContentPart> for OpenAITemplate {
    type Output = Option<serde_json::Value>;

    fn render(&self, value: &ChatMessageContentPart) -> nim_error::Result<Self::Output> {
        let text = |text: String| Ok(Some(serde_json::json!({ "type": "text", "text": text })));
        match value {
            ChatMessageContentPart::Text { text: value } => text(value.clone()),
            ChatMessageContentPart::Json { json } => text(serde_json::to_string(json)?),
            ChatMessageContentPart::Url { url } => text(url.to_string()),
            ChatMessageContentPart::Image(Media::Url { name, url, summary }) => Ok(Some(
                serde_json::json!({ "type": "image", "name": name, "image": url, "summary": summary }),
            )),
            ChatMessageContentPart::Image(Media::Path { name, path, summary }) => Ok(Some(
                serde_json::json!({ "type": "image", "name": name, "image": path, "summary": summary }),
            )),
            ChatMessageContentPart::Audio(Media::Url { name, url, summary }) => Ok(Some(
                serde_json::json!({ "type": "audio", "name": name, "image": url, "summary": summary }),
            )),
            ChatMessageContentPart::Audio(Media::Path { name, path, summary }) => Ok(Some(
                serde_json::json!({ "type": "audio", "name": name, "image": path, "summary": summary }),
            )),
            _ if self.compat == CompatMode::Ignore => Ok(None),
            _ => Err(nim_error::Diagnostic::message(
                "remote completion does not support this media content",
            )),
        }
    }
}

impl Render<Tool> for OpenAITemplate {
    type Output = serde_json::Value;

    fn render(&self, value: &Tool) -> nim_error::Result<Self::Output> {
        Ok(
            serde_json::json!({ "type": "function", "function": { "name": value.name, "description": value.description, "parameters": value.params } }),
        )
    }
}

impl Render<ToolCall> for OpenAITemplate {
    type Output = serde_json::Value;

    fn render(&self, value: &ToolCall) -> nim_error::Result<Self::Output> {
        Ok(
            serde_json::json!({ "id": value.id, "type": "function", "function": { "name": value.name, "arguments": serde_json::to_string(&value.args)? } }),
        )
    }
}
