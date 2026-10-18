use crate::models::GenOpts;
use crate::models::{ModelArgs, load};
use crate::types::{Artifact, ArtifactContent};
use crate::{Error, tasks};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub text: Vec<String>,
    pub model: ModelArgs,
    #[serde(default)]
    pub beams: Option<usize>,
    #[serde(default)]
    pub max_len: Option<usize>,
}

pub fn run(args: Args) -> Result<Vec<Artifact>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::generate())?, &api_key)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("generate"))?;
    let opts = GenOpts {
        beams: args.beams,
        max_len: args.max_len,
        ..GenOpts::default()
    };

    let out = tasks::summarize(
        capable,
        &model.context(),
        &args.text.iter().map(String::as_str).collect::<Vec<_>>(),
        &opts,
    )?;
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
