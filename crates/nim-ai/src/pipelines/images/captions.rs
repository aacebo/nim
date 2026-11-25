use crate::pipelines::text;
use crate::pipelines::text::embeddings::Options as EmbeddingOptions;
use crate::tasks;
use crate::types::Artifact;
use crate::{
    CaptionOptions,
    models::{ModelRef, load},
};

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
            model: super::defaults::captions(),
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
    let capable = model.as_image_caption().ok_or_else(|| model.cannot("caption"))?;
    let caption_options = CaptionOptions {
        beams: options.beams,
        max_tokens: options.max_tokens,
    };

    let captions = tasks::caption(capable, &model.context(), input, caption_options).await?;
    let out = text::embeddings::run_with(
        &captions.iter().map(|caption| caption.text.as_bytes()).collect::<Vec<_>>(),
        options.embedding,
    )
    .await?;
    let out = out
        .into_iter()
        .zip(captions)
        .map(|(mut artifact, caption)| {
            artifact.confidence = caption.confidence;
            artifact
        })
        .collect();

    Ok(out)
}
