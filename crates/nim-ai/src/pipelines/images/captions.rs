use crate::Error;
use crate::models::{ModelArgs, load};
use crate::pipelines::text;
use crate::types::Artifact;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub images: Vec<Vec<u8>>,
    pub model: ModelArgs,
    #[serde(default)]
    pub beams: Option<usize>,
    #[serde(default)]
    pub max_len: Option<usize>,
}

pub fn run(args: Args) -> Result<Vec<Artifact>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::captions())?, &api_key)?;
    let capable = model.as_image_to_text().ok_or_else(|| model.cannot("image-to-text"))?;
    let captions = capable.image_to_text(&model.context(), &args.images, args.beams, args.max_len)?;
    let out = text::embeddings::run(text::embeddings::Args {
        text: captions,
        ..Default::default()
    })?;

    Ok(out)
}
