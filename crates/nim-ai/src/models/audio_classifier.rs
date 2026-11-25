use crate::{Classify, ClassifyMode, ClassifyOptions, Context, types::Classification};

use super::{AnyModel, Loader, cnn8rnn};

pub enum AudioClassifier {
    Cnn8RnnClassifier(cnn8rnn::Cnn8RnnClassifier),
}

impl AudioClassifier {
    pub async fn load(repo: &Loader) -> nim_error::Result<Self> {
        let config: cnn8rnn::Cnn8RnnConfig = repo.config().await?;
        let labels = repo.string("classes.txt").await?.lines().map(str::to_string).collect();
        Ok(Self::Cnn8RnnClassifier(cnn8rnn::Cnn8RnnClassifier::new(
            config,
            labels,
            repo.vars().await?,
        )?))
    }

    pub fn mode(&self) -> ClassifyMode {
        match self {
            Self::Cnn8RnnClassifier(_) => ClassifyMode::Whole,
        }
    }
}

impl Classify for AudioClassifier {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        match self {
            Self::Cnn8RnnClassifier(model) => model.classify(ctx, input, options),
        }
    }
}

impl From<AudioClassifier> for AnyModel {
    fn from(model: AudioClassifier) -> Self {
        Self::AudioClassifier(model)
    }
}
