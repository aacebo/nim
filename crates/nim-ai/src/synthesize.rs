use crate::Context;

pub trait Synthesize: Send + Sync {
    fn synthesize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a str],
        options: SynthesizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<u8>>>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SpeechRate {
    Slow,
    #[default]
    Normal,
    Fast,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SynthesizeOptions {
    pub speed: Option<SpeechRate>,
    pub voice: Option<String>,
    pub language: Option<String>,
}

impl SynthesizeOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.voice.as_ref().is_some_and(|voice| voice.trim().is_empty()) {
            return Err(nim_error::Diagnostic::message("voice must not be empty"));
        }

        if self.language.as_ref().is_some_and(|language| language.trim().is_empty()) {
            return Err(nim_error::Diagnostic::message("language must not be empty"));
        }

        Ok(())
    }
}
