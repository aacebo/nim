use crate::Error;
use crate::models::{ModelArgs, load};
use crate::pipelines::text;
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

pub fn run(input: &[Vec<u8>]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

pub fn run_with(input: &[Vec<u8>], options: Options) -> Result<Vec<Artifact>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::transcribe())?, &api_key)?;
    let capable = model.as_transcribe().ok_or_else(|| model.cannot("transcribe"))?;
    let transcripts = capable.transcribe(
        &model.context(),
        input,
        options.language.as_deref(),
        options.beams,
        options.max_len,
    )?;
    let out = text::embeddings::run(&transcripts.iter().map(String::as_str).collect::<Vec<_>>())?;
    Ok(out)
}
