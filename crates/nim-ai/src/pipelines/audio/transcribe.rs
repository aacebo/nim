use crate::pipelines::text;
use crate::tasks;
use crate::types::Artifact;
use crate::{
    TranscribeOptions,
    models::{ModelRef, load},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
    pub language: Option<String>,
    pub beams: Option<usize>,
    pub max_tokens: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::transcribe(),
            language: None,
            beams: None,
            max_tokens: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub async fn run(input: &[&[u8]]) -> nim_error::Result<Vec<Artifact>> {
    run_with(input, Default::default()).await
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub async fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<Artifact>> {
    let model = load(&options.model).await?;
    let capable = model.as_transcribe().ok_or_else(|| model.cannot("transcribe"))?;
    let transcript_options = TranscribeOptions {
        beams: options.beams,
        max_tokens: options.max_tokens,
        language: options.language.as_deref(),
    };

    let transcripts = tasks::transcribe(capable, &model.context(), input, transcript_options).await?;
    let out = text::embeddings::run(
        &transcripts
            .iter()
            .map(|transcript| transcript.text.as_bytes())
            .collect::<Vec<_>>(),
    )
    .await?;

    let out = out
        .into_iter()
        .zip(transcripts)
        .map(|(mut artifact, transcript)| {
            artifact.confidence = transcript.confidence;
            artifact
        })
        .collect();

    Ok(out)
}
