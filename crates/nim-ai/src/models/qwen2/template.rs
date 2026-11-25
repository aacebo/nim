use nim_chat::{
    ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatSystemMessage, ChatToolMessage,
    ChatUserMessage, Document, Media, Tool, ToolCall,
};

use crate::Chat;

pub struct Qwen2Template;

impl Qwen2Template {
    pub fn new() -> Self {
        Self
    }
}

fn media_fields(
    name: &Option<String>,
    size: Option<usize>,
    mime_type: &Option<String>,
    summary: &Option<String>,
    source: &str,
    value: String,
) -> String {
    let mut fields = vec![];

    if let Some(name) = name {
        fields.push(format!("name={name}"));
    }

    if let Some(size) = size {
        fields.push(format!("size={size}"));
    }

    if let Some(mime_type) = mime_type {
        fields.push(format!("mime_type={mime_type}"));
    }

    if let Some(summary) = summary {
        fields.push(format!("summary={summary}"));
    }

    fields.push(format!("{source}={value}"));
    fields.join(" ")
}

impl nim_chat::Render<Chat> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(&'a self, _: &'a (), value: Chat) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let respond = matches!(value.render, crate::RenderMode::Respond)
                || matches!(value.render, crate::RenderMode::Auto)
                    && value
                        .messages
                        .last()
                        .is_none_or(|message| !matches!(message, ChatMessage::Agent(_)));

            let mut out = String::new();

            if !value.tools.is_empty() {
                out.push_str("<|im_start|>system\n<tools>\n");

                for tool in value.tools {
                    out.push_str(&self.render(&(), tool).await?);
                    out.push('\n');
                }

                out.push_str("</tools><|im_end|>\n");
            }

            if !value.documents.is_empty() {
                out.push_str("<|im_start|>system\n<documents>\n");

                for document in value.documents {
                    out.push_str(&self.render(&(), document).await?);
                    out.push('\n');
                }

                out.push_str("</documents><|im_end|>\n");
            }

            for message in value.messages {
                out.push_str(&self.render(&(), message).await?);
            }

            if respond {
                out.push_str("<|im_start|>assistant\n");
            }

            Ok(out)
        })
    }
}

impl nim_chat::Render<ChatMessage> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
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

impl nim_chat::Render<ChatSystemMessage> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatSystemMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            Ok(format!(
                "<|im_start|>system\n{}<|im_end|>\n",
                self.render(&(), value.content).await?
            ))
        })
    }
}

impl nim_chat::Render<ChatUserMessage> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatUserMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            Ok(format!(
                "<|im_start|>user\n{}<|im_end|>\n",
                self.render(&(), value.content).await?
            ))
        })
    }
}

impl nim_chat::Render<ChatAgentMessage> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatAgentMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let mut out = String::from("<|im_start|>assistant\n");

            if let Some(content) = value.content {
                out.push_str(&self.render(&(), content).await?);
            }

            for call in value.tool_calls {
                out.push_str(&self.render(&(), call).await?);
            }

            out.push_str("<|im_end|>\n");
            Ok(out)
        })
    }
}

impl nim_chat::Render<ChatToolMessage> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatToolMessage,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            Ok(format!(
                "<|im_start|>tool\n<tool_response id=\"{}\">{}</tool_response><|im_end|>\n",
                value.tool_call_id,
                match value.content {
                    Some(content) => self.render(&(), content).await?,
                    None => String::new(),
                }
            ))
        })
    }
}

impl nim_chat::Render<ChatMessageContent> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatMessageContent,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            match value {
                ChatMessageContent::String(value) => Ok(format!("{value}\n")),
                ChatMessageContent::Parts(values) => {
                    let mut out = String::new();

                    for value in values {
                        out.push_str(&self.render(&(), value).await?);
                    }

                    Ok(format!("{out}\n"))
                }
                _ => Err(nim_error::Diagnostic::message("unsupported message content")),
            }
        })
    }
}

impl nim_chat::Render<ChatMessageContentPart> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ChatMessageContentPart,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            match value {
                ChatMessageContentPart::Text { text } => Ok(text),
                ChatMessageContentPart::Thinking { thinking } => Ok(format!("<think>\n{thinking}\n</think>\n")),
                ChatMessageContentPart::Json { json } => Ok(serde_json::to_string(&json)?),
                ChatMessageContentPart::Url { url } => Ok(url.to_string()),
                ChatMessageContentPart::Image { media } => Ok(format!("[image: {}]", self.render(&(), media).await?)),
                ChatMessageContentPart::Audio { media } => Ok(format!("[audio: {}]", self.render(&(), media).await?)),
                ChatMessageContentPart::Video { media } => Ok(format!("[video: {}]", self.render(&(), media).await?)),
                _ => Err(nim_error::Diagnostic::message("unsupported message content part")),
            }
        })
    }
}

impl nim_chat::Render<Media> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(&'a self, _: &'a (), value: Media) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let value = &value;

            match value {
                Media::Path {
                    name,
                    path,
                    size,
                    mime_type,
                    summary,
                } => Ok(media_fields(
                    name,
                    *size,
                    mime_type,
                    summary,
                    "path",
                    path.display().to_string(),
                )),
                Media::Url {
                    name,
                    url,
                    size,
                    mime_type,
                    summary,
                } => Ok(media_fields(name, *size, mime_type, summary, "url", url.to_string())),
                _ => Err(nim_error::Diagnostic::message("unsupported media")),
            }
        })
    }
}

impl nim_chat::Render<Tool> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(&'a self, _: &'a (), value: Tool) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let value = &value;

            Ok(serde_json::to_string(value)?)
        })
    }
}

impl nim_chat::Render<ToolCall> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: ToolCall,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let value = &value;

            Ok(format!("<tool_call>{}</tool_call>\n", serde_json::to_string(value)?))
        })
    }
}

impl nim_chat::Render<Document> for Qwen2Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(
        &'a self,
        _: &'a (),
        value: Document,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let value = &value;

            Ok(serde_json::to_string(value)?)
        })
    }
}
