use crate::{Context, types::TextOutput};

pub trait Transcribe: Send + Sync {
    fn transcribe<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: TranscribeOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>>;
}

#[derive(Debug, Default, Clone)]
pub struct TranscribeOptions<'a> {
    pub language: Option<&'a str>,
    pub max_tokens: Option<usize>,
    pub beams: Option<usize>,
}

impl TranscribeOptions<'_> {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_tokens == Some(0) {
            return Err(nim_error::Diagnostic::message(
                "beams and max_tokens must be greater than zero",
            ));
        }

        Ok(())
    }
}
