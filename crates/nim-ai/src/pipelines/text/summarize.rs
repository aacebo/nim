use crate::models::GenerateOptions;
use crate::models::{ModelArgs, load};
use crate::pipelines::text;
use crate::types::Artifact;
use crate::{Error, tasks};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,

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
    let model = load(&options.model.resolve(super::defaults::generate())?)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("generate"))?;
    let generation_options = GenerateOptions {
        beams: options.beams,
        max_len: options.max_len,
        ..GenerateOptions::default()
    };

    let artifacts = tasks::summarize(capable, &model.context(), input, generation_options)?;
    let artifacts = text::embeddings::run(&artifacts.iter().map(String::as_bytes).collect::<Vec<_>>())?;
    let artifacts = artifacts
        .into_iter()
        .map(|artifact| Artifact {
            name: "summary".to_string(),
            value: artifact.value,
            vector: artifact.vector,
        })
        .collect();

    Ok(artifacts)
}
