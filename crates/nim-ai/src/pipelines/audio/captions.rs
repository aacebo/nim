use crate::models::{CaptionOptions, ModelRef, load};
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
            model: super::defaults::captions(),
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
    let capable = model.as_audio_caption().ok_or_else(|| model.cannot("caption"))?;
    let caption_options = CaptionOptions {
        beams: options.beams,
        max_len: options.max_len,
    };

    let captions = tasks::caption(capable, &model.context(), input, caption_options)?;
    let out = text::embeddings::run_with(
        &captions.iter().map(|caption| caption.text.as_bytes()).collect::<Vec<_>>(),
        options.embedding,
    )?;
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
