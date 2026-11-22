mod event;
mod io;
mod media;
mod message;
mod tool;

pub use event::*;
pub use io::*;
pub use media::*;
pub use message::*;
pub use tool::*;

use crate::Context;

pub trait Complete: Send + Sync {
    fn complete(&self, ctx: &Context, chat: Chat) -> nim_error::Result<ChatAssistantMessage>;
}

pub trait Parse<T>: Sized {
    fn parse(&self, p: Parser<'_>) -> nim_error::Result<T>;
}

pub trait Render<T> {
    type Output;

    fn render(&self, value: &T) -> nim_error::Result<Self::Output>;
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
pub struct Chat {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<Tool>,
    pub documents: Vec<Document>,
    pub thinking: bool,
    pub vision: bool,
    pub audio: bool,
    pub mode: RenderMode,
    pub compat: CompatMode,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl Chat {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
