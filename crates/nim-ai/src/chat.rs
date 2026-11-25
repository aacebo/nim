use crate::Context;

pub trait Complete: Send + Sync {
    fn complete<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        chat: Chat,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<nim_chat::ChatAgentMessage>>;
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Chat {
    pub messages: Vec<nim_chat::ChatMessage>,
    pub tools: Vec<nim_chat::Tool>,
    pub documents: Vec<nim_chat::Document>,
    pub thinking: bool,
    pub vision: bool,
    pub audio: bool,
    pub render: RenderMode,
    pub parse: nim_chat::ParseMode,
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
