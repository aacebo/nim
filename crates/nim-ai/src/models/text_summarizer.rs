use std::sync::Arc;

use crate::{Context, Summarize, SummarizeOptions, types::TextOutput};

use super::{AnyModel, Loader, bart, qwen2, qwen3};

pub enum TextSummarizer {
    BartSummarizer(bart::BartSummarizer),
    Qwen2Generator(Arc<qwen2::Qwen2Generator>),
    Qwen3Generator(Arc<qwen3::Qwen3Generator>),
}

impl TextSummarizer {
    pub async fn load(repo: &Loader) -> nim_error::Result<Self> {
        let config: bart::BartConfig = repo.config().await?;
        Ok(Self::BartSummarizer(bart::BartSummarizer::new(&config, repo.vars().await?)?))
    }
}

impl Summarize for TextSummarizer {
    fn summarize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: SummarizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        match self {
            Self::BartSummarizer(model) => model.summarize(ctx, input, options),
            Self::Qwen2Generator(model) => model.summarize(ctx, input, options),
            Self::Qwen3Generator(model) => model.summarize(ctx, input, options),
        }
    }
}

impl From<TextSummarizer> for AnyModel {
    fn from(model: TextSummarizer) -> Self {
        Self::TextSummarizer(model)
    }
}
