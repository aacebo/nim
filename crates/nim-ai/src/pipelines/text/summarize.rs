use crate::models::GenOpts;
use crate::models::{ModelArgs, load};
use crate::types::{Artifact, ArtifactContent};
use crate::{Error, tasks};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

pub fn run(input: &[&str]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

pub fn run_with(input: &[&str], options: Options) -> Result<Vec<Artifact>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::generate())?, &api_key)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("generate"))?;
    let opts = GenOpts {
        beams: options.beams,
        max_len: options.max_len,
        ..GenOpts::default()
    };

    let out = tasks::summarize(capable, &model.context(), input, &opts)?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .map(|summary| Artifact {
            name: "summary".to_string(),
            value: ArtifactContent::text(summary),
            vector: None,
        })
        .collect();

    Ok(artifacts)
}
