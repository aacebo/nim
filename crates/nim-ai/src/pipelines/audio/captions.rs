use crate::Error;
use crate::models::{ModelArgs, load};
use crate::types::{Artifact, ArtifactContent};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,

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
    let model = load(&options.model.resolve(super::defaults::captions())?, &api_key)?;
    let capable = model.as_audio_to_text().ok_or_else(|| model.cannot("audio-to-text"))?;
    let captions = capable.audio_to_text(&model.context(), input, options.beams, options.max_len)?;

    Ok(captions
        .into_iter()
        .map(|caption| Artifact {
            name: "caption".to_string(),
            value: ArtifactContent::text(caption),
            vector: None,
        })
        .collect())
}
