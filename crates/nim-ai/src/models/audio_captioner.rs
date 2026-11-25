use crate::{Caption, CaptionOptions, Context, types::TextOutput};

use super::{AnyModel, Loader, ModelConfig, audiocap, whisper};

pub enum AudioCaptioner {
    WhisperCaptioner(whisper::WhisperCaptioner),
    AudioCapCaptioner(audiocap::AudioCapCaptioner),
}

impl AudioCaptioner {
    pub async fn load(repo: &Loader, config: &ModelConfig) -> nim_error::Result<Self> {
        if config.is_cnn14_audio_captioning() {
            let config: audiocap::AudioCapConfig = repo.config().await?;
            Ok(Self::AudioCapCaptioner(audiocap::AudioCapCaptioner::new(
                config,
                repo.vars().await?,
            )?))
        } else {
            let config: whisper::WhisperConfig = repo.config().await?;
            let processor: whisper::WhisperProcessor = repo.json("preprocessor_config.json").await?;
            Ok(Self::WhisperCaptioner(whisper::WhisperCaptioner::new(
                config,
                processor,
                repo.vars().await?,
            )?))
        }
    }
}

impl Caption for AudioCaptioner {
    fn caption<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: CaptionOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        match self {
            Self::WhisperCaptioner(model) => model.caption(ctx, input, options),
            Self::AudioCapCaptioner(model) => model.caption(ctx, input, options),
        }
    }
}

impl From<AudioCaptioner> for AnyModel {
    fn from(model: AudioCaptioner) -> Self {
        Self::AudioCaptioner(model)
    }
}
