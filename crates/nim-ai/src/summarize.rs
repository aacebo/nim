use crate::{Context, types::TextOutput};

pub trait Summarize: Send + Sync {
    fn summarize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: SummarizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>>;
}

#[derive(Debug, Default, Clone)]
pub struct SummarizeOptions {
    pub max_tokens: Option<usize>,
    pub beams: Option<usize>,
}

impl SummarizeOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_tokens == Some(0) {
            return Err(nim_error::Diagnostic::message(
                "beams and max_tokens must be greater than zero",
            ));
        }

        Ok(())
    }
}
