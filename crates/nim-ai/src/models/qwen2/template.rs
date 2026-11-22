use crate::{
    Chat, ChatAssistantMessage, ChatMessage, ChatMessageContent, ChatSystemMessage, ChatToolMessage, ChatUserMessage, CompatMode,
    Document, Parse, Parser, Render, Tool, ToolCall,
};

pub struct Qwen2Template {
    template: hf_chat_template::ChatTemplate,
}

impl Qwen2Template {
    pub fn new(template: hf_chat_template::ChatTemplate) -> Self {
        Self { template }
    }

    pub fn render_chat(&self, chat: &Chat) -> nim_error::Result<String> {
        self.render(chat)
    }
}

impl Render<Chat> for Qwen2Template {
    type Output = String;

    fn render(&self, value: &Chat) -> nim_error::Result<String> {
        let mut value = value.clone();

        if matches!(value.mode, crate::RenderMode::Continue) && value.compat == CompatMode::Error {
            return Err(nim_error::Diagnostic::message(
                "local completion does not support continuation mode",
            ));
        }

        if value
            .messages
            .iter()
            .any(|message| message.has_unsupported_parts(value.vision, value.audio))
        {
            if value.compat == CompatMode::Error {
                return Err(nim_error::Diagnostic::message(
                    "local Qwen completion does not support requested media content",
                ));
            }

            for message in &mut value.messages {
                message.retain_supported_parts(value.vision, value.audio);
            }
        }

        let messages: Vec<hf_chat_template::Message> = value
            .messages
            .into_iter()
            .map(|value| self.render(&value))
            .collect::<nim_error::Result<_>>()?;
        let tools: Vec<serde_json::Value> = value
            .tools
            .into_iter()
            .map(|value| self.render(&value))
            .collect::<nim_error::Result<_>>()?;
        let documents: Vec<serde_json::Value> = value
            .documents
            .into_iter()
            .map(|value| self.render(&value))
            .collect::<nim_error::Result<_>>()?;
        let add_generation_prompt = match value.mode {
            crate::RenderMode::None => false,
            crate::RenderMode::Respond => true,
            crate::RenderMode::Continue => false,
            crate::RenderMode::Auto => messages.last().map(|value| value.role != "assistant").unwrap_or_default(),
        };
        let mut extra = serde_json::Map::new();

        if value.vision {
            extra.insert("vision".into(), true.into());
        }

        if value.audio {
            extra.insert("audio".into(), true.into());
        }

        if value.thinking {
            extra.insert("enable_thinking".into(), true.into());
        }

        let input = hf_chat_template::RenderInput {
            messages,
            tools,
            documents,
            add_generation_prompt,
            extra,
        };
        self.template.render(&input).map_err(nim_error::Diagnostic::message)
    }
}

impl Render<ChatMessage> for Qwen2Template {
    type Output = hf_chat_template::Message;

    fn render(&self, value: &ChatMessage) -> nim_error::Result<Self::Output> {
        match value {
            ChatMessage::System(value) => self.render(value),
            ChatMessage::User(value) => self.render(value),
            ChatMessage::Assistant(value) => self.render(value),
            ChatMessage::Tool(value) => self.render(value),
        }
    }
}

impl Render<ChatSystemMessage> for Qwen2Template {
    type Output = hf_chat_template::Message;

    fn render(&self, value: &ChatSystemMessage) -> nim_error::Result<Self::Output> {
        Ok(hf_chat_template::Message {
            role: "system".into(),
            content: Some(self.render(&value.content)?),
            ..Default::default()
        })
    }
}

impl Render<ChatUserMessage> for Qwen2Template {
    type Output = hf_chat_template::Message;

    fn render(&self, value: &ChatUserMessage) -> nim_error::Result<Self::Output> {
        Ok(hf_chat_template::Message {
            role: "user".into(),
            content: Some(self.render(&value.content)?),
            ..Default::default()
        })
    }
}

impl Render<ChatAssistantMessage> for Qwen2Template {
    type Output = hf_chat_template::Message;

    fn render(&self, value: &ChatAssistantMessage) -> nim_error::Result<Self::Output> {
        let mut extra = serde_json::Map::new();

        if let Some(reasoning) = &value.reasoning {
            extra.extend([
                ("reasoning".into(), serde_json::to_value(reasoning)?),
                ("reasoning_content".into(), serde_json::to_value(reasoning)?),
                ("thinking".into(), serde_json::to_value(reasoning)?),
            ]);
        }

        Ok(hf_chat_template::Message {
            role: "assistant".into(),
            content: value.content.as_ref().map(|value| self.render(value)).transpose()?,
            tool_calls: value
                .tool_calls
                .iter()
                .map(|value| self.render(value))
                .collect::<nim_error::Result<_>>()?,
            extra,
        })
    }
}

impl Render<ChatToolMessage> for Qwen2Template {
    type Output = hf_chat_template::Message;

    fn render(&self, value: &ChatToolMessage) -> nim_error::Result<Self::Output> {
        Ok(hf_chat_template::Message {
            role: "tool".into(),
            content: value.content.as_ref().map(|value| self.render(value)).transpose()?,
            extra: [("tool_call_id".into(), serde_json::to_value(&value.tool_call_id)?)]
                .into_iter()
                .collect(),
            ..Default::default()
        })
    }
}

impl Render<ChatMessageContent> for Qwen2Template {
    type Output = hf_chat_template::Content;

    fn render(&self, value: &ChatMessageContent) -> nim_error::Result<Self::Output> {
        match value {
            ChatMessageContent::Text(value) => Ok(hf_chat_template::Content::Text(value.clone())),
            ChatMessageContent::Parts(value) => Ok(hf_chat_template::Content::Parts(
                value.iter().map(serde_json::to_value).collect::<Result<_, _>>()?,
            )),
        }
    }
}

impl Render<Tool> for Qwen2Template {
    type Output = serde_json::Value;

    fn render(&self, value: &Tool) -> nim_error::Result<Self::Output> {
        Ok(
            serde_json::json!({ "type": "function", "function": { "name": value.name, "description": value.description, "parameters": value.params } }),
        )
    }
}

impl Render<ToolCall> for Qwen2Template {
    type Output = serde_json::Value;

    fn render(&self, value: &ToolCall) -> nim_error::Result<Self::Output> {
        Ok(serde_json::json!({ "id": value.id, "type": "function", "function": { "name": value.name, "arguments": value.args } }))
    }
}

impl Render<Document> for Qwen2Template {
    type Output = serde_json::Value;

    fn render(&self, value: &Document) -> nim_error::Result<Self::Output> {
        Ok(serde_json::to_value(value)?)
    }
}

impl Parse<ChatMessage> for Qwen2Template {
    fn parse(&self, parser: Parser<'_>) -> nim_error::Result<ChatMessage> {
        let message: ChatAssistantMessage = self.parse(parser)?;

        Ok(message.into())
    }
}

impl Parse<Chat> for Qwen2Template {
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

impl Parse<ChatAssistantMessage> for Qwen2Template {
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

impl Parse<ToolCall> for Qwen2Template {
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
