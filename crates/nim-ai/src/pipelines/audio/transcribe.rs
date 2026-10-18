use crate::Error;
use crate::models::{ModelArgs, load};
use crate::pipelines::text;
use crate::types::Artifact;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub audio: Vec<Vec<u8>>,
    pub model: ModelArgs,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub beams: Option<usize>,
    #[serde(default)]
    pub max_len: Option<usize>,
}

pub fn run(args: Args) -> Result<Vec<Artifact>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::transcribe())?, &api_key)?;
    let capable = model.as_transcribe().ok_or_else(|| model.cannot("transcribe"))?;
    let transcripts = capable.transcribe(
        &model.context(),
        &args.audio,
        args.language.as_deref(),
        args.beams,
        args.max_len,
    )?;

    let out = text::embeddings::run(text::embeddings::Args {
        text: transcripts,
        ..Default::default()
    })?;

    Ok(out)
}
