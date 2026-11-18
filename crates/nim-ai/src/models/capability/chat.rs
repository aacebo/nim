use crate::models::Context;
use crate::types::TextOutput;

pub trait Chat: Send + Sync {
    fn chat(&self, ctx: &Context, input: &[&[u8]], options: ChatOptions<'_>) -> nim_error::Result<Vec<TextOutput>>;
}

#[derive(Debug, Default, Clone)]
pub struct ChatOptions<'a> {
    pub prompt: Option<&'a str>,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl ChatOptions<'_> {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
