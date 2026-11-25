use crate::SummarizeOptions;
use crate::models::{ModelRef, load};
use crate::pipelines::text;
use crate::pipelines::text::embeddings::Options as EmbeddingOptions;
use crate::tasks;
use crate::types::Artifact;

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
    pub embedding: EmbeddingOptions,
    pub beams: Option<usize>,
    pub max_tokens: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::summarize(),
            embedding: Default::default(),
            beams: None,
            max_tokens: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub async fn run(input: &[&[u8]]) -> nim_error::Result<Vec<Artifact>> {
    run_with(input, Default::default()).await
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub async fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<Artifact>> {
    let model = load(&options.model).await?;
    let capable = model.as_summarize().ok_or_else(|| model.cannot("summarize"))?;
    let summary_options = SummarizeOptions {
        beams: options.beams,
        max_tokens: options.max_tokens,
    };

    let summaries = tasks::summarize(capable, &model.context(), input, summary_options).await?;
    let artifacts = text::embeddings::run_with(
        &summaries.iter().map(|summary| summary.text.as_bytes()).collect::<Vec<_>>(),
        options.embedding,
    )
    .await?;
    let artifacts = artifacts
        .into_iter()
        .zip(summaries)
        .map(|(artifact, summary)| Artifact {
            name: "summary".to_string(),
            value: artifact.value,
            vector: artifact.vector,
            confidence: summary.confidence,
        })
        .collect();

    Ok(artifacts)
}
