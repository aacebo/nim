use nim_chat::{
    ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatSystemMessage, ChatToolMessage,
    ChatUserMessage, Media, ParseMode, Tool, ToolCall,
};

use crate::Chat;

pub struct OpenAITemplate {
    parse: ParseMode,
}

impl OpenAITemplate {
    pub fn new(parse: ParseMode) -> Self {
        Self { parse }
    }
}

impl nim_chat::Render<Chat> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Vec<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(&'a self, _: &'a (), value: Chat) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let mut messages = Vec::new();

            for document in value.documents {
                let mut message = hf_chat_template::Message::system(document.content.clone());
                message.extra.insert("document".into(), serde_json::to_value(document)?);
                let document = message
                    .extra
                    .remove("document")
                    .ok_or_else(|| nim_error::Diagnostic::message("document message is missing document metadata"))?;
                messages.push(serde_json::json!({
                    "role": "system",
                    "content": serde_json::to_string(&document)?,
                }));
            }

            for message in value.messages {
                if let Some(message) = self.render(&(), message).await? {
                    messages.push(message);
                }
            }

            Ok(messages)
        })
    }
}

impl nim_chat::Render<ChatMessage> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            match value {
                ChatMessage::System(value) => self.render(&(), value).await,
                ChatMessage::User(value) => self.render(&(), value).await,
                ChatMessage::Agent(value) => self.render(&(), value).await,
                ChatMessage::Tool(value) => self.render(&(), value).await,
                _ => Err(nim_error::Diagnostic::message("unsupported message role")),
            }
        })
    }
}

impl nim_chat::Render<ChatSystemMessage> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatSystemMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            Ok(self
                .render(&(), value.content)
                .await?
                .map(|content| serde_json::json!({ "role": "system", "content": content })))
        })
    }
}

impl nim_chat::Render<ChatUserMessage> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatUserMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            Ok(self
                .render(&(), value.content)
                .await?
                .map(|content| serde_json::json!({ "role": "user", "content": content })))
        })
    }
}

impl nim_chat::Render<ChatAgentMessage> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatAgentMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let mut thinking = Vec::new();
            let content = match value.content {
                Some(ChatMessageContent::Parts(parts)) => {
                    let mut content = Vec::new();

                    for part in parts {
                        match part {
                            ChatMessageContentPart::Thinking { thinking: value } => thinking.push(value),
                            other => content.push(other),
                        }
                    }

                    if content.is_empty() {
                        None
                    } else {
                        Some(ChatMessageContent::Parts(content))
                    }
                }
                content => content,
            };

            let content = match content {
                Some(content) => Some(self.render(&(), content).await?),
                None => None,
            };

            let mut tool_calls = Vec::new();

            for call in value.tool_calls {
                tool_calls.push(self.render(&(), call).await?);
            }

            let mut message = serde_json::json!({
                "role": "assistant",
                "content": content,
                "tool_calls": tool_calls,
            });

            if !thinking.is_empty() {
                message["reasoning_content"] = serde_json::Value::String(thinking.join("\n"));
            }

            Ok(Some(message))
        })
    }
}

impl nim_chat::Render<ChatToolMessage> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatToolMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let content = match value.content {
                Some(content) => Some(self.render(&(), content).await?),
                None => None,
            };

            Ok(Some(serde_json::json!({
                "role": "tool",
                "tool_call_id": value.tool_call_id,
                "content": content,
            })))
        })
    }
}

impl nim_chat::Render<ChatMessageContent> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatMessageContent,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            match value {
                ChatMessageContent::String(text) => Ok(Some(serde_json::Value::String(text))),
                ChatMessageContent::Parts(parts) => {
                    let mut rendered = Vec::new();

                    for part in parts {
                        if let Some(part) = self.render(&(), part).await? {
                            rendered.push(part);
                        }
                    }

                    if rendered.is_empty() && self.parse == ParseMode::Lenient {
                        Ok(None)
                    } else if rendered.is_empty() {
                        Err(nim_error::Diagnostic::message("message has no supported content"))
                    } else {
                        Ok(Some(serde_json::Value::Array(rendered)))
                    }
                }
                _ => Err(nim_error::Diagnostic::message("unsupported message content")),
            }
        })
    }
}

impl nim_chat::Render<ChatMessageContentPart> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = Option<serde_json::Value>;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatMessageContentPart,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let text = |text: String| Ok(Some(serde_json::json!({ "type": "text", "text": text })));

            match value {
                ChatMessageContentPart::Text { text: value } => text(value),
                ChatMessageContentPart::Json { json } => text(serde_json::to_string(&json)?),
                ChatMessageContentPart::Url { url } => text(url.to_string()),
                ChatMessageContentPart::Image {
                    media:
                        Media::Url {
                            name,
                            url,
                            size,
                            mime_type,
                            summary,
                        },
                } => Ok(Some(serde_json::json!({
                    "type": "image",
                    "name": name,
                    "size": size,
                    "mime_type": mime_type,
                    "image": url,
                    "summary": summary,
                }))),
                ChatMessageContentPart::Image {
                    media:
                        Media::Path {
                            name,
                            path,
                            size,
                            mime_type,
                            summary,
                        },
                } => Ok(Some(serde_json::json!({
                    "type": "image",
                    "name": name,
                    "size": size,
                    "mime_type": mime_type,
                    "image": path,
                    "summary": summary,
                }))),
                ChatMessageContentPart::Video {
                    media:
                        Media::Url {
                            name,
                            url,
                            size,
                            mime_type,
                            summary,
                        },
                } => Ok(Some(serde_json::json!({
                    "type": "video",
                    "name": name,
                    "size": size,
                    "mime_type": mime_type,
                    "video": url,
                    "summary": summary,
                }))),
                ChatMessageContentPart::Video {
                    media:
                        Media::Path {
                            name,
                            path,
                            size,
                            mime_type,
                            summary,
                        },
                } => Ok(Some(serde_json::json!({
                    "type": "video",
                    "name": name,
                    "size": size,
                    "mime_type": mime_type,
                    "video": path,
                    "summary": summary,
                }))),
                ChatMessageContentPart::Audio {
                    media:
                        Media::Url {
                            name,
                            url,
                            size,
                            mime_type,
                            summary,
                        },
                } => Ok(Some(serde_json::json!({
                    "type": "audio",
                    "name": name,
                    "size": size,
                    "mime_type": mime_type,
                    "audio": url,
                    "summary": summary,
                }))),
                ChatMessageContentPart::Audio {
                    media:
                        Media::Path {
                            name,
                            path,
                            size,
                            mime_type,
                            summary,
                        },
                } => Ok(Some(serde_json::json!({
                    "type": "audio",
                    "name": name,
                    "size": size,
                    "mime_type": mime_type,
                    "audio": path,
                    "summary": summary,
                }))),
                _ if self.parse == ParseMode::Lenient => Ok(None),
                _ => Err(nim_error::Diagnostic::message("unsupported message content part")),
            }
        })
    }
}

impl nim_chat::Render<Tool> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = serde_json::Value;
    type Context<'a> = ();

    fn render<'a>(&'a self, _: &'a (), value: Tool) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let value = &value;

            Ok(serde_json::json!({
                "type": "function",
                "function": {
                    "name": value.name,
                    "description": value.description,
                    "parameters": value.params,
                },
            }))
        })
    }
}

impl nim_chat::Render<ToolCall> for OpenAITemplate {
    type Error = nim_error::Diagnostic;
    type Output = serde_json::Value;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ToolCall,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let value = &value;

            Ok(serde_json::json!({
                "id": value.id,
                "type": "function",
                "function": {
                    "name": value.name,
                    "arguments": serde_json::to_string(&value.args)?,
                },
            }))
        })
    }
}
