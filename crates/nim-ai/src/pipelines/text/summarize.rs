use crate::models::SummarizeOptions;
use crate::models::{ModelRef, load};
use crate::pipelines::text;
use crate::pipelines::text::embeddings::Options as EmbeddingOptions;
use crate::tasks;
use crate::types::Artifact;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,

    #[serde(default)]
    pub embedding: EmbeddingOptions,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::summarize(),
            embedding: Default::default(),
            beams: None,
            max_len: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> nim_error::Result<Vec<Artifact>> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<Artifact>> {
    let model = load(&options.model)?;
    let capable = model.as_summarize().ok_or_else(|| model.cannot("summarize"))?;
    let summary_options = SummarizeOptions {
        beams: options.beams,
        max_len: options.max_len,
    };

    let summaries = tasks::summarize(capable, &model.context(), input, summary_options)?;
    let artifacts = text::embeddings::run_with(
        &summaries.iter().map(|summary| summary.text.as_bytes()).collect::<Vec<_>>(),
        options.embedding,
    )?;
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
