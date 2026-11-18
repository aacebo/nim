use crate::models::Context;
use crate::types::TextOutput;

pub trait Caption: Send + Sync {
    fn caption(&self, ctx: &Context, input: &[&[u8]], options: CaptionOptions) -> nim_error::Result<Vec<TextOutput>>;
}

#[derive(Debug, Default, Clone)]
pub struct CaptionOptions {
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl CaptionOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
