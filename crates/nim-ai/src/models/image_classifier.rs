use crate::{Classify, ClassifyMode, ClassifyOptions, Context, types::Classification};

use super::{AnyModel, Loader, ModelType, detr, mobilenet};

pub enum ImageClassifier {
    MobileNetClassifier(mobilenet::MobileNetClassifier),
    DetrDetector(detr::DetrDetector),
}

impl ImageClassifier {
    pub async fn load(repo: &Loader, model_type: ModelType) -> nim_error::Result<Self> {
        match model_type {
            ModelType::MobileNetV2 => {
                let config: mobilenet::MobileNetConfig = repo.config().await?;
                let processor: mobilenet::MobileNetProcessor = repo.json("preprocessor_config.json").await?;
                Ok(Self::MobileNetClassifier(mobilenet::MobileNetClassifier::new(
                    repo.vars().await?,
                    config,
                    processor,
                )?))
            }
            ModelType::Detr => {
                let config: detr::DetrConfig = repo.config().await?;
                config.validate()?;
                let processor: detr::DetrProcessor = repo.json("preprocessor_config.json").await?;
                Ok(Self::DetrDetector(detr::DetrDetector::new(
                    repo.vars().await?,
                    config,
                    processor,
                )?))
            }
            _ => Err(nim_error::Diagnostic::message(format!(
                "unsupported image classification architecture `{model_type}`"
            ))),
        }
    }

    pub fn mode(&self) -> ClassifyMode {
        match self {
            Self::MobileNetClassifier(_) => ClassifyMode::Whole,
            Self::DetrDetector(_) => ClassifyMode::Regions,
        }
    }
}

impl Classify for ImageClassifier {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        match self {
            Self::MobileNetClassifier(model) => model.classify(ctx, input, options),
            Self::DetrDetector(model) => model.classify(ctx, input, options),
        }
    }
}

impl From<ImageClassifier> for AnyModel {
    fn from(model: ImageClassifier) -> Self {
        Self::ImageClassifier(model)
    }
}
