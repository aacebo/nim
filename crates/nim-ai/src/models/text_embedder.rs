use std::sync::Arc;

use crate::{Context, Embed, EmbedOptions};

use super::{AnyModel, Loader, ModelType, bert, embeddinggemma, qwen3};

pub enum TextEmbedder {
    BertEmbedder(Arc<bert::KeyBert<bert::BertEmbedder>>),
    Qwen3Embedder(qwen3::Qwen3Embedder),
    EmbeddingGemmaEmbedder(embeddinggemma::EmbeddingGemmaEmbedder),
}

impl TextEmbedder {
    pub async fn load(repo: &Loader, model_type: ModelType) -> nim_error::Result<Self> {
        match model_type {
            ModelType::Bert | ModelType::Unknown => {
                let config: bert::BertConfig = repo.config().await?;
                let model = bert::BertEmbedder::new(repo.vars().await?, &config)?;
                Ok(Self::BertEmbedder(Arc::new(bert::KeyBert::new(model))))
            }
            ModelType::Qwen3 => Ok(Self::Qwen3Embedder(qwen3::Qwen3Embedder::new(repo).await?)),
            ModelType::Gemma3Text => Ok(Self::EmbeddingGemmaEmbedder(
                embeddinggemma::EmbeddingGemmaEmbedder::new(repo).await?,
            )),
            _ => Err(nim_error::Diagnostic::message(format!(
                "unsupported text embedding architecture `{model_type}`"
            ))),
        }
    }
}

impl Embed for TextEmbedder {
    fn embed<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        match self {
            Self::BertEmbedder(model) => model.embed(ctx, input),
            Self::Qwen3Embedder(model) => model.embed(ctx, input),
            Self::EmbeddingGemmaEmbedder(model) => model.embed(ctx, input),
        }
    }

    fn embed_with<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: &'a EmbedOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        match self {
            Self::BertEmbedder(model) => model.embed_with(ctx, input, options),
            Self::Qwen3Embedder(model) => model.embed_with(ctx, input, options),
            Self::EmbeddingGemmaEmbedder(model) => model.embed_with(ctx, input, options),
        }
    }
}

impl From<TextEmbedder> for AnyModel {
    fn from(model: TextEmbedder) -> Self {
        Self::TextEmbedder(model)
    }
}
