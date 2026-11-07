use crate::models::{EmbedOptions, ModelRef, load};
use crate::types::{Artifact, ArtifactContent};
use crate::{Error, tasks};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,
    #[serde(default)]
    pub embedding: EmbedOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::embed(),
            embedding: Default::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Artifact>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Artifact>, Error> {
    let text = input
        .iter()
        .map(|bytes| std::str::from_utf8(bytes).map_err(|err| Error::custom("ai::encoding", err)))
        .collect::<Result<Vec<_>, _>>()?;
    let model = load(&options.model)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::embed_with(capable, &model.context(), input, &options.embedding)?;
    let artifacts: Vec<Artifact> = out
        .into_iter()
        .zip(text)
        .map(|(vector, text)| Artifact {
            name: "embedding".to_string(),
            value: ArtifactContent::text(text),
            vector: Some(vector),
            confidence: None,
        })
        .collect();

    Ok(artifacts)
}
