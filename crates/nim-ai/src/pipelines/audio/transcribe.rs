use crate::Error;
use crate::models::{GenerateOptions, ModelRef, load};
use crate::pipelines::text;
use crate::tasks;
use crate::types::Artifact;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,

    #[serde(default)]
    pub language: Option<String>,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::transcribe(),
            language: None,
            beams: None,
            max_len: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Artifact>, Error> {
    let model = load(&options.model)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("generate"))?;
    let generation_options = GenerateOptions {
        beams: options.beams,
        max_len: options.max_len,
        language: options.language.as_deref(),
        ..Default::default()
    };

    let transcripts = tasks::generate(capable, &model.context(), input, generation_options)?;
    let out = text::embeddings::run(
        &transcripts
            .iter()
            .map(|generation| generation.text.as_bytes())
            .collect::<Vec<_>>(),
    )?;

    let out = out
        .into_iter()
        .zip(transcripts)
        .map(|(mut artifact, generation)| {
            artifact.confidence = generation.confidence;
            artifact
        })
        .collect();

    Ok(out)
}
