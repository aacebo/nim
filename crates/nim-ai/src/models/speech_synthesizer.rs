use crate::{Context, Synthesize, SynthesizeOptions};

use super::{AnyModel, Loader, parler};

pub enum SpeechSynthesizer {
    ParlerSynthesizer(parler::ParlerSynthesizer),
}

impl SpeechSynthesizer {
    pub async fn load(repo: &Loader, dtype: candle_core::DType) -> nim_error::Result<Self> {
        let config = parler::ParlerConfig::new(repo).await?;
        let processor = parler::ParlerProcessor::new(repo, &config, dtype).await?;
        Ok(Self::ParlerSynthesizer(parler::ParlerSynthesizer::new(
            config,
            processor,
            repo.vars().await?,
        )?))
    }
}

impl Synthesize for SpeechSynthesizer {
    fn synthesize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a str],
        options: SynthesizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<u8>>> {
        match self {
            Self::ParlerSynthesizer(model) => model.synthesize(ctx, input, options),
        }
    }
}

impl From<SpeechSynthesizer> for AnyModel {
    fn from(model: SpeechSynthesizer) -> Self {
        Self::SpeechSynthesizer(model)
    }
}
