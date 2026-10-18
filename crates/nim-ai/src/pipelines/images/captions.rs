use crate::Error;
use crate::models::{ModelArgs, load};
use crate::pipelines::text;
use crate::types::Artifact;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[Vec<u8>]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[Vec<u8>], options: Options) -> Result<Vec<Artifact>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::captions())?, &api_key)?;
    let capable = model.as_image_to_text().ok_or_else(|| model.cannot("image-to-text"))?;
    let captions = capable.image_to_text(&model.context(), input, options.beams, options.max_len)?;
    let out = text::embeddings::run(&captions.iter().map(String::as_str).collect::<Vec<_>>())?;
    Ok(out)
}
