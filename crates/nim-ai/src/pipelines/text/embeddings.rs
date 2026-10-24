use crate::models::{ModelArgs, load};
use crate::pipelines::common::decode_text;
use crate::types::{Artifact, ArtifactContent};
use crate::{Error, tasks};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Artifact>, Error> {
    let text = decode_text(input)?;
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::embed())?, &api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::embed(capable, &model.context(), input)?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .zip(text)
        .map(|(vector, text)| Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
        })
        .collect();

    Ok(artifacts)
}
