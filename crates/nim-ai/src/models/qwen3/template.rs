use crate::{
    Chat, ChatAssistantMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatSystemMessage, ChatToolMessage,
    ChatUserMessage, CompatMode, Document, Media, Parse, Parser, Render, Tool, ToolCall,
};

pub struct Qwen3Template;

impl Qwen3Template {
    pub fn new() -> Self {
        Self
    }

    pub fn render_chat(&self, chat: &Chat) -> nim_error::Result<String> {
        self.render(chat)
    }
}

impl Render<Chat> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &Chat) -> nim_error::Result<String> {
        let mut out = String::new();

        if !value.tools.is_empty() {
            out.push_str("<|im_start|>system\n<tools>\n");

            for tool in &value.tools {
                out.push_str(&self.render(tool)?);
                out.push('\n');
            }

            out.push_str("</tools><|im_end|>\n");
        }

        if !value.documents.is_empty() {
            out.push_str("<|im_start|>system\n<documents>\n");

            for document in &value.documents {
                out.push_str(&self.render(document)?);
                out.push('\n');
            }

            out.push_str("</documents><|im_end|>\n");
        }

        for message in &value.messages {
            out.push_str(&self.render(message)?);
        }

        let respond = matches!(value.mode, crate::RenderMode::Respond)
            || matches!(value.mode, crate::RenderMode::Auto)
                && value
                    .messages
                    .last()
                    .is_none_or(|message| !matches!(message, ChatMessage::Assistant(_)));

        if respond {
            out.push_str("<|im_start|>assistant\n");
        }

        Ok(out)
    }
}

impl Render<ChatMessage> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatMessage) -> nim_error::Result<String> {
        match value {
            ChatMessage::System(value) => self.render(value),
            ChatMessage::User(value) => self.render(value),
            ChatMessage::Assistant(value) => self.render(value),
            ChatMessage::Tool(value) => self.render(value),
        }
    }
}

impl Render<ChatSystemMessage> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatSystemMessage) -> nim_error::Result<String> {
        Ok(format!("<|im_start|>system\n{}<|im_end|>\n", self.render(&value.content)?))
    }
}

impl Render<ChatUserMessage> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatUserMessage) -> nim_error::Result<String> {
        Ok(format!("<|im_start|>user\n{}<|im_end|>\n", self.render(&value.content)?))
    }
}

impl Render<ChatAssistantMessage> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatAssistantMessage) -> nim_error::Result<String> {
        let mut out = String::from("<|im_start|>assistant\n");

        if let Some(reasoning) = &value.reasoning {
            out.push_str("<think>\n");
            out.push_str(reasoning);
            out.push_str("\n</think>\n");
        }

        if let Some(content) = &value.content {
            out.push_str(&self.render(content)?);
        }

        for call in &value.tool_calls {
            out.push_str(&self.render(call)?);
        }

        out.push_str("<|im_end|>\n");
        Ok(out)
    }
}

impl Render<ChatToolMessage> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatToolMessage) -> nim_error::Result<String> {
        Ok(format!(
            "<|im_start|>tool\n<tool_response id=\"{}\">{}</tool_response><|im_end|>\n",
            value.tool_call_id,
            value
                .content
                .as_ref()
                .map(|value| self.render(value))
                .transpose()?
                .unwrap_or_default()
        ))
    }
}

impl Render<ChatMessageContent> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatMessageContent) -> nim_error::Result<String> {
        match value {
            ChatMessageContent::Text(value) => Ok(value.clone()),
            ChatMessageContent::Parts(values) => values.iter().map(|value| self.render(value)).collect(),
        }
    }
}

impl Render<ChatMessageContentPart> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ChatMessageContentPart) -> nim_error::Result<String> {
        match value {
            ChatMessageContentPart::Text { text } => Ok(text.clone()),
            ChatMessageContentPart::Json { json } => Ok(serde_json::to_string(json)?),
            ChatMessageContentPart::Url { url } => Ok(url.to_string()),
            ChatMessageContentPart::Image(value) => Ok(format!("[image: {}]", self.render(value)?)),
            ChatMessageContentPart::Audio(value) => Ok(format!("[audio: {}]", self.render(value)?)),
            ChatMessageContentPart::Video(value) => Ok(format!("[video: {}]", self.render(value)?)),
        }
    }
}

impl Render<Media> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &Media) -> nim_error::Result<String> {
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
        }
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

impl Render<Tool> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &Tool) -> nim_error::Result<String> {
        Ok(serde_json::to_string(value)?)
    }
}

impl Render<ToolCall> for Qwen3Template {
    type Output = String;

    fn render(&self, value: &ToolCall) -> nim_error::Result<String> {
        Ok(format!("<tool_call>{}</tool_call>\n", serde_json::to_string(value)?))
    }
}

impl Render<Document> for Qwen3Template {
    type Output = String;
    fn render(&self, value: &Document) -> nim_error::Result<String> {
        Ok(serde_json::to_string(value)?)
    }
}

impl Parse<ChatMessage> for Qwen3Template {
    fn parse(&self, parser: Parser<'_>) -> nim_error::Result<ChatMessage> {
        let message: ChatAssistantMessage = self.parse(parser)?;

        Ok(message.into())
    }
}

impl Parse<Chat> for Qwen3Template {
    fn parse(&self, parser: Parser<'_>) -> nim_error::Result<Chat> {
        let compat = parser.compat();
        let message: ChatMessage = self.parse(parser)?;

        Ok(Chat {
            messages: vec![message],
            compat,
            ..Default::default()
        })
    }
}

impl Parse<ChatAssistantMessage> for Qwen3Template {
    fn parse(&self, mut parser: Parser<'_>) -> nim_error::Result<ChatAssistantMessage> {
        let mut reasoning = None;

        if parser.take_prefix("<think>") {
            match parser.take_until("</think>") {
                Some(value) => reasoning = Some(value.trim().to_string()),
                None if parser.compat() == CompatMode::Error => {
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
                if parser.compat() == CompatMode::Error {
                    return Err(nim_error::Diagnostic::message("unterminated Qwen tool call"));
                }

                content.push_str("<tool_call>");
                content.push_str(parser.take_remaining());
                break;
            };

            let parsed: nim_error::Result<ToolCall> = self.parse(Parser::new(raw.trim(), parser.compat()));

            match parsed {
                Ok(mut call) => {
                    if call.id.is_empty() {
                        call.id = format!("call_{}", tool_calls.len());
                    }

                    tool_calls.push(call);
                }
                Err(error) if parser.compat() == CompatMode::Error => return Err(error),
                Err(_) => {
                    content.push_str("<tool_call>");
                    content.push_str(raw);
                    content.push_str("</tool_call>");
                }
            }
        }

        content.push_str(parser.take_remaining());
        let content = (!content.trim().is_empty()).then(|| ChatMessageContent::Text(content.trim().to_string()));

        Ok(ChatAssistantMessage {
            content,
            reasoning,
            tool_calls,
            annotations: Vec::new(),
        })
    }
}

impl Parse<ToolCall> for Qwen3Template {
    fn parse(&self, mut parser: Parser<'_>) -> nim_error::Result<ToolCall> {
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
    }
}
