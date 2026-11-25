use std::sync::Arc;

use crate::{Chat, Complete, Context};

use super::{AnyModel, Loader, ModelType, qwen2, qwen3};

pub enum TextCompleter {
    Qwen2Generator(Arc<qwen2::Qwen2Generator>),
    Qwen3Generator(Arc<qwen3::Qwen3Generator>),
}

impl TextCompleter {
    pub async fn load(repo: &Loader, model_type: ModelType) -> nim_error::Result<Self> {
        match model_type {
            ModelType::Qwen2 => {
                let config: candle_transformers::models::qwen2::Config = repo.config().await?;
                Ok(Self::Qwen2Generator(Arc::new(qwen2::Qwen2Generator::new(
                    &config,
                    repo.vars().await?,
                )?)))
            }
            ModelType::Qwen3 => {
                let config: candle_transformers::models::qwen3::Config = repo.config().await?;
                Ok(Self::Qwen3Generator(Arc::new(qwen3::Qwen3Generator::new(
                    &config,
                    repo.vars().await?,
                )?)))
            }
            _ => Err(nim_error::Diagnostic::message(format!(
                "unsupported text completion architecture `{model_type}`"
            ))),
        }
    }

    pub async fn quantized(repo: &Loader, companion: &Loader, model_type: ModelType, weights: &str) -> nim_error::Result<Self> {
        match model_type {
            ModelType::Qwen2 => {
                let config: candle_transformers::models::qwen2::Config = companion.config().await?;
                let weights = repo.resolve(std::path::Path::new(weights)).await?;
                Ok(Self::Qwen2Generator(Arc::new(qwen2::Qwen2Generator::quantized(
                    &config,
                    weights,
                    repo.device(),
                )?)))
            }
            ModelType::Qwen3 => {
                let config: candle_transformers::models::qwen3::Config = companion.config().await?;
                let weights = repo.resolve(std::path::Path::new(weights)).await?;
                Ok(Self::Qwen3Generator(Arc::new(qwen3::Qwen3Generator::quantized(
                    &config,
                    weights,
                    repo.device(),
                )?)))
            }
            _ => Err(nim_error::Diagnostic::message(format!(
                "unsupported text completion architecture `{model_type}`"
            ))),
        }
    }
}

impl Complete for TextCompleter {
    fn complete<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        chat: Chat,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<nim_chat::ChatAgentMessage>> {
        match self {
            Self::Qwen2Generator(model) => model.complete(ctx, chat),
            Self::Qwen3Generator(model) => model.complete(ctx, chat),
        }
    }
}

impl From<TextCompleter> for AnyModel {
    fn from(model: TextCompleter) -> Self {
        Self::TextCompleter(model)
    }
}
