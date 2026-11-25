use crate::Context;

pub trait Embed: Send + Sync {
    fn embed<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>>;

    fn embed_with<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: &'a EmbedOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            options.validate()?;

            if options != &EmbedOptions::default() {
                return Err(nim_error::Diagnostic::message("model does not support embedding options"));
            }

            self.embed(ctx, input).await
        })
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EmbedMode {
    #[default]
    Raw,
    Query,
    Document,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EmbedOptions {
    pub mode: EmbedMode,
    pub instruction: Option<String>,
    pub dimensions: Option<usize>,
}

impl EmbedOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.instruction.is_some() && self.mode != EmbedMode::Query {
            return Err(nim_error::Diagnostic::message("embedding instructions require query mode"));
        }

        Ok(())
    }
}
