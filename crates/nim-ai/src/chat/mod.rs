mod event;
mod media;
mod message;
mod tool;

pub use event::*;
pub use media::*;
pub use message::*;
pub use tool::*;

use crate::Context;

pub trait Chat: Send + Sync {
    fn chat(&self, ctx: &Context, options: ChatOptions) -> nim_error::Result<ChatAssistantMessage>;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatMode {
    /// error when unsupported fields/messages are provided
    #[default]
    Error,

    /// ignore unsupported fields/messages
    Ignore,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMode {
    /// continue if the last message is from the assistant, otherwise respond
    #[default]
    Auto,

    /// append a new assistant generation prefix
    Respond,

    /// continue the final message without its ending delimiter
    Continue,

    /// render the conversation without generation prefix
    None,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatOptions {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<Tool>,
    pub documents: Vec<Document>,
    pub thinking: bool,
    pub vision: bool,
    pub mode: RenderMode,
    pub compat: CompatMode,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl ChatOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}

impl TryFrom<ChatOptions> for hf_chat_template::RenderInput {
    type Error = nim_error::Diagnostic;

    fn try_from(mut value: ChatOptions) -> Result<Self, Self::Error> {
        if matches!(value.mode, RenderMode::Continue) && value.compat == CompatMode::Error {
            return Err(nim_error::Diagnostic::message(
                "local chat does not support continuation mode",
            ));
        }

        if value.vision && value.compat == CompatMode::Error {
            return Err(nim_error::Diagnostic::message(
                "local Qwen chat does not support vision input",
            ));
        }

        if value.messages.iter().any(ChatMessage::has_non_text_parts) {
            if value.compat == CompatMode::Error {
                return Err(nim_error::Diagnostic::message(
                    "local Qwen chat does not support media content",
                ));
            }

            for message in &mut value.messages {
                message.retain_text_parts();
            }
        }

        let add_generation_prompt = match value.mode {
            RenderMode::None => false,
            RenderMode::Respond => true,
            RenderMode::Continue => false,
            RenderMode::Auto => value.messages.last().map(|v| v.role() != "assistant").unwrap_or_default(),
        };

        let (messages, errors) =
            value
                .messages
                .into_iter()
                .map(|v| v.into_hf())
                .fold((Vec::new(), Vec::new()), |(mut values, mut errors), result| {
                    match result {
                        Ok(value) => values.push(value),
                        Err(error) => errors.push(error.source().clone()),
                    }
                    (values, errors)
                });

        if !errors.is_empty() {
            return Err(nim_error::Diagnostic::new().append(errors));
        }

        let (tools, errors) =
            value
                .tools
                .into_iter()
                .map(|v| v.into_hf())
                .fold((Vec::new(), Vec::new()), |(mut values, mut errors), result| {
                    match result {
                        Ok(value) => values.push(value),
                        Err(error) => errors.push(error.source().clone()),
                    }
                    (values, errors)
                });

        if !errors.is_empty() {
            return Err(nim_error::Diagnostic::new().append(errors));
        }

        let (documents, errors) = value.documents.into_iter().map(serde_json::to_value).fold(
            (Vec::new(), Vec::new()),
            |(mut values, mut errors), result| {
                match result {
                    Ok(value) => values.push(value),
                    Err(error) => errors.push(error),
                }
                (values, errors)
            },
        );

        if !errors.is_empty() {
            return Err(nim_error::Diagnostic::new().append(errors));
        }

        let mut extra = serde_json::Map::new();

        if value.vision {
            extra.insert("vision".into(), true.into());
        }

        if value.thinking {
            extra.insert("enable_thinking".into(), true.into());
        }

        Ok(Self {
            messages,
            tools,
            documents,
            add_generation_prompt,
            extra,
        })
    }
}
