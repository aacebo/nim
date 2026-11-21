use crate::{Context, types::TextOutput};

pub trait Summarize: Send + Sync {
    fn summarize(&self, ctx: &Context, input: &[&[u8]], options: SummarizeOptions) -> nim_error::Result<Vec<TextOutput>>;
}

#[derive(Debug, Default, Clone)]
pub struct SummarizeOptions {
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl SummarizeOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
