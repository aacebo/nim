use crate::Chat;

pub mod chatml;

pub trait Format {
    type Error;

    fn parse(&self, src: &str) -> Result<Chat, Self::Error>;
    fn render(&self, chat: &Chat) -> Result<String, Self::Error>;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum CompatMode {
    /// reject unsupported content
    #[default]
    Error,

    /// discard unsupported content
    Ignore,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
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
