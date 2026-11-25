use crate::{Context, Transcribe, TranscribeOptions, types::TextOutput};

use super::{AnyModel, Loader, whisper};

pub enum AudioTranscriber {
    WhisperTranscriber(whisper::WhisperTranscriber),
}

impl AudioTranscriber {
    pub async fn load(repo: &Loader) -> nim_error::Result<Self> {
        let config: whisper::WhisperConfig = repo.config().await?;
        let processor: whisper::WhisperProcessor = repo.json("preprocessor_config.json").await?;
        Ok(Self::WhisperTranscriber(whisper::WhisperTranscriber::new(
            config,
            processor,
            repo.vars().await?,
        )?))
    }
}

impl Transcribe for AudioTranscriber {
    fn transcribe<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: TranscribeOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        match self {
            Self::WhisperTranscriber(model) => model.transcribe(ctx, input, options),
        }
    }
}

impl From<AudioTranscriber> for AnyModel {
    fn from(model: AudioTranscriber) -> Self {
        Self::AudioTranscriber(model)
    }
}
