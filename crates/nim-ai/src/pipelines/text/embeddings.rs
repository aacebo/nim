use crate::models::{ModelArgs, load};
use crate::types::{Artifact, ArtifactContent};
use crate::{Error, tasks};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub text: Vec<String>,
    pub model: ModelArgs,
}

pub fn run(args: Args) -> Result<Vec<Artifact>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::embed())?, &api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::embed(
        capable,
        &model.context(),
        &args.text.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .zip(args.text)
        .map(|(vector, text)| Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
        })
        .collect();

    Ok(artifacts)
}
