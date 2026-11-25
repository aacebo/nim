use crate::{Context, types::TextOutput};

pub trait Caption: Send + Sync {
    fn caption<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: CaptionOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>>;
}

#[derive(Debug, Default, Clone)]
pub struct CaptionOptions {
    pub max_tokens: Option<usize>,
    pub beams: Option<usize>,
}

impl CaptionOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_tokens == Some(0) {
            return Err(nim_error::Diagnostic::message(
                "beams and max_tokens must be greater than zero",
            ));
        }

        Ok(())
    }
}
