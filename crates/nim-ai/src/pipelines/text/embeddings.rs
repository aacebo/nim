use crate::tasks;
use crate::types::{Artifact, ArtifactContent};
use crate::{
    EmbedOptions,
    models::{ModelRef, load},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
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
pub async fn run(input: &[&[u8]]) -> nim_error::Result<Vec<Artifact>> {
    run_with(input, Default::default()).await
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub async fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<Artifact>> {
    let text = input
        .iter()
        .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
        .collect::<Result<Vec<_>, _>>()?;
    let model = load(&options.model).await?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::embed_with(capable, &model.context(), input, &options.embedding).await?;
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
