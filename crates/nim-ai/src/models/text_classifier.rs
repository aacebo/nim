use std::sync::Arc;

use crate::{Classify, ClassifyMode, ClassifyOptions, Context, types::Classification};

use super::{AnyModel, Loader, ModelConfig, ModelType, bart, bert, distilbert};

pub enum TextClassifier {
    BartEntityClassifier(bart::BartClassifier),
    BartSequenceClassifier(bart::BartClassifier),
    BartZeroShotClassifier(bart::BartZeroShotClassifier),
    BertEntityClassifier(bert::BertClassifier),
    BertSequenceClassifier(bert::BertClassifier),
    BertEmbedder(Arc<bert::KeyBert<bert::BertEmbedder>>),
    DistilBertEntityClassifier(distilbert::DistilBertClassifier),
    DistilBertSequenceClassifier(distilbert::DistilBertClassifier),
}

impl TextClassifier {
    pub async fn load(repo: &Loader, config: &ModelConfig, name: &str) -> nim_error::Result<Self> {
        let sequence = config.is_sequence_classification();
        let token = config.is_token_classification();

        match config.model_type {
            ModelType::Bart => {
                let config: bart::BartConfig = repo.config().await?;

                if sequence && bart::BartZeroShotClassifier::supports(&config) {
                    Ok(Self::BartZeroShotClassifier(bart::BartZeroShotClassifier::new(
                        &config,
                        repo.vars().await?,
                    )?))
                } else if sequence {
                    Ok(Self::BartSequenceClassifier(bart::BartClassifier::new(
                        &config,
                        repo.vars().await?,
                        ClassifyMode::Whole,
                    )?))
                } else {
                    Ok(Self::BartEntityClassifier(bart::BartClassifier::new(
                        &config,
                        repo.vars().await?,
                        ClassifyMode::Regions,
                    )?))
                }
            }
            ModelType::DistilBert => {
                let config: distilbert::DistilBertConfig = repo.config().await?;

                if sequence && !config.id2label.is_empty() {
                    Ok(Self::DistilBertSequenceClassifier(distilbert::DistilBertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Whole,
                    )?))
                } else if token && !config.id2label.is_empty() {
                    Ok(Self::DistilBertEntityClassifier(distilbert::DistilBertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Regions,
                    )?))
                } else {
                    Err(nim_error::Diagnostic::message(format!(
                        "{name} has no compatible classification head"
                    )))
                }
            }
            ModelType::Bert | ModelType::Unknown => {
                let config: bert::BertConfig = repo.config().await?;

                if sequence {
                    Ok(Self::BertSequenceClassifier(bert::BertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Whole,
                    )?))
                } else {
                    Ok(Self::BertEntityClassifier(bert::BertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Regions,
                    )?))
                }
            }
            model_type => Err(nim_error::Diagnostic::message(format!(
                "unsupported text classification architecture `{model_type}`"
            ))),
        }
    }

    pub fn mode(&self) -> ClassifyMode {
        match self {
            Self::BartEntityClassifier(_) => ClassifyMode::Regions,
            Self::BartSequenceClassifier(_) => ClassifyMode::Whole,
            Self::BartZeroShotClassifier(_) => ClassifyMode::Regions,
            Self::BertEntityClassifier(_) => ClassifyMode::Regions,
            Self::BertSequenceClassifier(_) => ClassifyMode::Whole,
            Self::BertEmbedder(_) => ClassifyMode::Regions,
            Self::DistilBertEntityClassifier(_) => ClassifyMode::Regions,
            Self::DistilBertSequenceClassifier(_) => ClassifyMode::Whole,
        }
    }
}

impl Classify for TextClassifier {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        match self {
            Self::BartEntityClassifier(model) => model.classify(ctx, input, options),
            Self::BartSequenceClassifier(model) => model.classify(ctx, input, options),
            Self::BartZeroShotClassifier(model) => model.classify(ctx, input, options),
            Self::BertEntityClassifier(model) => model.classify(ctx, input, options),
            Self::BertSequenceClassifier(model) => model.classify(ctx, input, options),
            Self::BertEmbedder(model) => model.classify(ctx, input, options),
            Self::DistilBertEntityClassifier(model) => model.classify(ctx, input, options),
            Self::DistilBertSequenceClassifier(model) => model.classify(ctx, input, options),
        }
    }
}

impl From<TextClassifier> for AnyModel {
    fn from(model: TextClassifier) -> Self {
        Self::TextClassifier(model)
    }
}
