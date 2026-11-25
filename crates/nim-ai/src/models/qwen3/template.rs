use nim_chat::{
    ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatSystemMessage, ChatToolMessage,
    ChatUserMessage, Document, Media, Parser, Tool, ToolCall,
};

use crate::{Chat, CompatMode};

pub struct Qwen3Template;

impl Qwen3Template {
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

impl nim_chat::Render<Chat> for Qwen3Template {
    type Error = nim_error::Diagnostic;
    type Output = String;
    type Context<'a> = ();

    fn render<'a>(&'a self, _: &'a (), value: Chat) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>> {
        Box::pin(async move {
            let respond = matches!(value.mode, crate::RenderMode::Respond)
                || matches!(value.mode, crate::RenderMode::Auto)
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

impl nim_chat::Render<ChatMessage> for Qwen3Template {
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

impl nim_chat::Render<ChatSystemMessage> for Qwen3Template {
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

impl nim_chat::Render<ChatUserMessage> for Qwen3Template {
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

impl nim_chat::Render<ChatAgentMessage> for Qwen3Template {
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

            if let Some(reasoning) = &value.reasoning {
                out.push_str("<think>\n");
                out.push_str(reasoning);
                out.push_str("\n</think>\n");
            }

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

impl nim_chat::Render<ChatToolMessage> for Qwen3Template {
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

impl nim_chat::Render<ChatMessageContent> for Qwen3Template {
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
                ChatMessageContent::String(value) => Ok(value),
                ChatMessageContent::Parts(values) => {
                    let mut out = String::new();

                    for value in values {
                        out.push_str(&self.render(&(), value).await?);
                    }

                    Ok(out)
                }
                _ => Err(nim_error::Diagnostic::message("unsupported message content")),
            }
        })
    }
}

impl nim_chat::Render<ChatMessageContentPart> for Qwen3Template {
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
                ChatMessageContentPart::Json { json } => Ok(serde_json::to_string(&json)?),
                ChatMessageContentPart::Url { url } => Ok(url.to_string()),
                ChatMessageContentPart::Image { media: value } => Ok(format!("[image: {}]", self.render(&(), value).await?)),
                ChatMessageContentPart::Audio { media: value } => Ok(format!("[audio: {}]", self.render(&(), value).await?)),
                ChatMessageContentPart::Video { media: value } => Ok(format!("[video: {}]", self.render(&(), value).await?)),
                _ => Err(nim_error::Diagnostic::message("unsupported message content part")),
            }
        })
    }
}

impl nim_chat::Render<Media> for Qwen3Template {
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

impl nim_chat::Render<Tool> for Qwen3Template {
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

impl nim_chat::Render<ToolCall> for Qwen3Template {
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

impl nim_chat::Render<Document> for Qwen3Template {
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

impl nim_chat::Parse<ChatMessage> for Qwen3Template {
    type Error = nim_error::Diagnostic;
    type Context<'a> = Parser<'a>;

    fn parse<'a>(&'a self, parser: &'a Parser<'a>) -> futures_util::future::BoxFuture<'a, Result<ChatMessage, Self::Error>> {
        Box::pin(async move {
            let message: ChatAgentMessage = self.parse(parser).await?;

            Ok(message.into())
        })
    }
}

impl nim_chat::Parse<Chat> for Qwen3Template {
    type Error = nim_error::Diagnostic;
    type Context<'a> = Parser<'a>;

    fn parse<'a>(&'a self, parser: &'a Parser<'a>) -> futures_util::future::BoxFuture<'a, Result<Chat, Self::Error>> {
        Box::pin(async move {
            let compat = if parser.ignore_unsupported() {
                CompatMode::Ignore
            } else {
                CompatMode::Error
            };

            let message: ChatMessage = self.parse(parser).await?;

            Ok(Chat {
                messages: vec![message],
                compat,
                ..Default::default()
            })
        })
    }
}

impl nim_chat::Parse<ChatAgentMessage> for Qwen3Template {
    type Error = nim_error::Diagnostic;
    type Context<'a> = Parser<'a>;

    fn parse<'a>(&'a self, parser: &'a Parser<'a>) -> futures_util::future::BoxFuture<'a, Result<ChatAgentMessage, Self::Error>> {
        Box::pin(async move {
            let mut parser = Parser::new(parser.remaining(), parser.ignore_unsupported());

            let mut reasoning = None;

            if parser.take_prefix("<think>") {
                match parser.take_until("</think>") {
                    Some(value) => reasoning = Some(value.trim().to_string()),
                    None if !parser.ignore_unsupported() => {
                        return Err(nim_error::Diagnostic::message("unterminated Qwen reasoning block"));
                    }
                    None => {}
                }
            }

            let mut content = String::new();
            let mut tool_calls = Vec::new();

            while let Some(value) = parser.take_until("<tool_call>") {
                content.push_str(value);

                let Some(raw) = parser.take_until("</tool_call>") else {
                    if !parser.ignore_unsupported() {
                        return Err(nim_error::Diagnostic::message("unterminated Qwen tool call"));
                    }

                    content.push_str("<tool_call>");
                    content.push_str(parser.take_remaining());
                    break;
                };

                let parsed: nim_error::Result<ToolCall> = self.parse(&Parser::new(raw.trim(), parser.ignore_unsupported())).await;

                match parsed {
                    Ok(mut call) => {
                        if call.id.is_empty() {
                            call.id = format!("call_{}", tool_calls.len());
                        }

                        tool_calls.push(call);
                    }
                    Err(error) if !parser.ignore_unsupported() => return Err(error),
                    Err(_) => {
                        content.push_str("<tool_call>");
                        content.push_str(raw);
                        content.push_str("</tool_call>");
                    }
                }
            }

            content.push_str(parser.take_remaining());
            let content = (!content.trim().is_empty()).then(|| ChatMessageContent::String(content.trim().to_string()));

            Ok(ChatAgentMessage {
                name: None,
                content,
                reasoning,
                tool_calls,
                annotations: Vec::new(),
                extra: Default::default(),
            })
        })
    }
}

impl nim_chat::Parse<ToolCall> for Qwen3Template {
    type Error = nim_error::Diagnostic;
    type Context<'a> = Parser<'a>;

    fn parse<'a>(&'a self, parser: &'a Parser<'a>) -> futures_util::future::BoxFuture<'a, Result<ToolCall, Self::Error>> {
        Box::pin(async move {
            let mut parser = Parser::new(parser.remaining(), parser.ignore_unsupported());

            let value: serde_json::Value = serde_json::from_str(parser.take_remaining())?;
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
                .unwrap_or_default();

            Ok(ToolCall { id, name, args })
        })
    }
}
