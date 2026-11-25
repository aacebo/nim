use crate::Context;

pub trait Synthesize: Send + Sync {
    fn synthesize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a str],
        options: SynthesizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<u8>>>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SynthesizeOptions {
    pub speed: Option<f32>,
    pub voice: Option<String>,
    pub language: Option<String>,
}

impl SynthesizeOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.speed.is_some_and(|speed| !speed.is_finite() || speed <= 0.0) {
            return Err(nim_error::Diagnostic::message("speed must be finite and greater than zero"));
        }

        Ok(())
    }
}
