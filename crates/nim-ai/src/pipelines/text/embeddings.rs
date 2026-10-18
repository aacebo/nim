use crate::models::{ModelArgs, load};
use crate::types::{Artifact, ArtifactContent};
use crate::{Error, tasks};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
}

pub fn run(input: &[&str]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

pub fn run_with(input: &[&str], options: Options) -> Result<Vec<Artifact>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::embed())?, &api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::embed(capable, &model.context(), input)?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .zip(input)
        .map(|(vector, text)| Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(*text),
            vector: Some(vector),
        })
        .collect();

    Ok(artifacts)
}
