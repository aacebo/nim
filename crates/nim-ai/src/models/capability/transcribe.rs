use crate::models::Context;
use crate::types::TextOutput;

pub trait Transcribe: Send + Sync {
    fn transcribe(&self, ctx: &Context, input: &[&[u8]], options: TranscribeOptions<'_>) -> nim_error::Result<Vec<TextOutput>>;
}

#[derive(Debug, Default, Clone)]
pub struct TranscribeOptions<'a> {
    pub language: Option<&'a str>,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl TranscribeOptions<'_> {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
