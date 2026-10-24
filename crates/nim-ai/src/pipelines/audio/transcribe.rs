use crate::Error;
use crate::models::{GenerateOptions, ModelArgs, load};
use crate::pipelines::text;
use crate::tasks;
use crate::types::Artifact;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,

    #[serde(default)]
    pub language: Option<String>,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Artifact>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::transcribe())?, &api_key)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("generate"))?;
    let generation_options = GenerateOptions {
        beams: options.beams,
        max_len: options.max_len,
        language: options.language.as_deref(),
        ..Default::default()
    };
    let transcripts = tasks::generate(capable, &model.context(), input, generation_options)?;
    let out = text::embeddings::run(&transcripts.iter().map(String::as_bytes).collect::<Vec<_>>())?;
    Ok(out)
}
