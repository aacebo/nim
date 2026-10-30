#[cfg(feature = "audio")]
pub mod audiocap;
pub mod bart;
pub mod bert;
#[cfg(feature = "audio")]
pub mod cnn8rnn;
#[cfg(feature = "vision")]
pub mod detr;
pub mod distilbert;
#[cfg(feature = "vision")]
pub mod git;
pub mod keybert;
#[cfg(feature = "vision")]
pub mod mobilenet;
#[cfg(feature = "audio")]
pub mod whisper;

mod architecture;
mod cache;
mod capability;
mod loader;
mod model_args;
mod model_config;
mod model_id;
mod model_ref;
mod model_type;
mod provider;
mod remote;
mod tokenizer;

pub use architecture::*;
use cache::*;
pub use capability::*;
pub use loader::*;
pub use model_args::*;
pub use model_config::*;
pub use model_id::*;
pub use model_ref::*;
pub use model_type::*;
pub use provider::*;

use candle_core::{DType, Device};

use crate::Error;
use crate::resources::Repository;

static MODELS: std::sync::LazyLock<Cache<crate::models::Model>> = std::sync::LazyLock::new(Cache::new);
type LoadCallback = dyn Fn(&ModelRef, &ModelConfig) + Send + Sync + 'static;

static LOAD_CALLBACKS: std::sync::LazyLock<std::sync::RwLock<std::collections::HashMap<u64, std::sync::Arc<LoadCallback>>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));
static NEXT_SUBSCRIPTION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn id(value: impl AsRef<str>) -> Result<ModelId, Error> {
    value.as_ref().parse::<ModelId>()
}

pub fn refer(value: impl AsRef<str>) -> Result<ModelRef, Error> {
    value.as_ref().parse::<ModelRef>()
}

/// One cache of loaded models, keyed by `(model, api_key)` -- not one per capability. A model used
/// for two routines now loads its weights once.
#[tracing::instrument]
pub fn load(model: &ModelRef) -> Result<std::sync::Arc<crate::models::Model>, Error> {
    use candle_core::{DType, Device};

    MODELS.get_or_build(Key::new(model), || {
        if let ModelRef::Remote(_) = model {
            return Ok(std::sync::Arc::new(Model::new(model, Device::Cpu, DType::F32)?));
        }

        let callbacks = LOAD_CALLBACKS.read()?.values().cloned().collect::<Vec<_>>();
        let loaded = Model::new(model, Device::Cpu, DType::F32)?;

        if let Some(config) = &loaded.config {
            for callback in &callbacks {
                callback(model, config);
            }
        }

        Ok(std::sync::Arc::new(loaded))
    })
}

pub fn on_load(callback: impl Fn(&ModelRef, &ModelConfig) + Send + Sync + 'static) -> Result<crate::Subscription, Error> {
    let id = NEXT_SUBSCRIPTION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    LOAD_CALLBACKS.write()?.insert(id, std::sync::Arc::new(callback));

    Ok(crate::Subscription::new(move || {
        LOAD_CALLBACKS
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&id);
    }))
}

pub trait Forward: Send + Sync {
    type Input;
    type Output;

    fn forward(&self, input: Self::Input) -> Result<Self::Output, Error>;
}

#[derive(Debug, Copy, Clone)]
pub enum ClassifyMedia {
    Text,
    #[cfg(feature = "vision")]
    Image,
    #[cfg(feature = "audio")]
    Audio,
}

/// A model with its weights loaded (or, for a remote model, its endpoint bound). The variant
/// determines which capabilities exist: the `None` arms below are the empty cells of the matrix.
pub enum AnyModel {
    Embedder(keybert::KeyBert<bert::Embedder>),
    BartEntityClassifier(bart::Classifier),
    BartSequenceClassifier(bart::Classifier),
    BartZeroShotClassifier(bart::ZeroShotClassifier),
    BertEntityClassifier(bert::Classifier),
    BertSequenceClassifier(bert::Classifier),
    DistilBertEntityClassifier(distilbert::Classifier),
    DistilBertSequenceClassifier(distilbert::Classifier),
    Summarizer(bart::Summarizer),
    #[cfg(feature = "vision")]
    VisionClassifier(mobilenet::Classifier),
    #[cfg(feature = "vision")]
    ObjectDetector(detr::Detector),
    #[cfg(feature = "vision")]
    ImageCaptioner(git::Captioner),
    #[cfg(feature = "audio")]
    AudioClassifier(cnn8rnn::Classifier),
    #[cfg(feature = "audio")]
    Transcriber(whisper::Transcriber),
    #[cfg(feature = "audio")]
    WhisperCaptioner(whisper::Captioner),
    #[cfg(feature = "audio")]
    AudioCaptioner(audiocap::Captioner),
    Remote(RemoteModel),
}

/// A loaded model plus the tokenizer and device it needs to see text. Lends a [`Context`] per
/// call, so one set of weights serves every capability the model has.
pub struct Model {
    name: String,
    model: AnyModel,
    device: Device,
    tokenizer: Option<tokenizers::Tokenizer>,
    config: Option<ModelConfig>,
}

impl Model {
    pub fn new(model: &ModelRef, device: Device, dtype: DType) -> Result<Self, Error> {
        let name = model.to_string();

        match model {
            ModelRef::Remote(remote) => Ok(Self {
                name,
                model: AnyModel::Remote(remote.clone()),
                device,
                tokenizer: None,
                config: None,
            }),
            ModelRef::Local(local) => Self::local(local, device, dtype, name),
        }
    }

    pub fn local(model: &LocalModel, device: Device, dtype: DType, name: String) -> Result<Self, Error> {
        let repo = model.loader(device, dtype)?;
        let device = repo.device().clone();
        let model_config: ModelConfig = repo.config()?;

        let model = match model_config.model_type {
            #[cfg(feature = "audio")]
            _ if model_config.is_cnn8rnn_audio_classification() => {
                let config: cnn8rnn::Config = repo.config()?;
                let labels = repo.string("classes.txt")?.lines().map(str::to_string).collect();
                AnyModel::AudioClassifier(cnn8rnn::Classifier::new(config, labels, repo.vars()?)?)
            }
            #[cfg(feature = "audio")]
            _ if model_config.is_cnn14_audio_captioning() => {
                let config: audiocap::Config = repo.config()?;
                AnyModel::AudioCaptioner(audiocap::Captioner::new(config, repo.vars()?)?)
            }
            #[cfg(not(feature = "audio"))]
            _ if model_config.architectures.iter().any(|architecture| {
                matches!(
                    architecture,
                    Architecture::Cnn8RnnSoundEventDetection | Architecture::Cnn14RnnTempAttnGruModel
                )
            }) =>
            {
                return Err(Error::custom("ai::config", format!("{name} requires the `audio` feature")));
            }
            #[cfg(not(feature = "vision"))]
            ModelType::MobileNetV2 | ModelType::Git | ModelType::Detr => {
                return Err(Error::custom("ai::config", format!("{name} requires the `vision` feature")));
            }
            #[cfg(not(feature = "audio"))]
            ModelType::Whisper => {
                return Err(Error::custom("ai::config", format!("{name} requires the `audio` feature")));
            }
            ModelType::Bart => {
                let config: bart::Config = repo.config()?;

                match (
                    model_config.is_sequence_classification(),
                    model_config.is_token_classification(),
                    config.id2label.is_empty(),
                ) {
                    (true, _, false) if bart::ZeroShotClassifier::supports(&config) => {
                        AnyModel::BartZeroShotClassifier(bart::ZeroShotClassifier::new(&config, repo.vars()?)?)
                    }
                    (true, _, false) => {
                        AnyModel::BartSequenceClassifier(bart::Classifier::new(&config, repo.vars()?, ClassifyMode::Whole)?)
                    }
                    (_, true, false) => {
                        AnyModel::BartEntityClassifier(bart::Classifier::new(&config, repo.vars()?, ClassifyMode::Regions)?)
                    }
                    _ => AnyModel::Summarizer(bart::Summarizer::new(&config, repo.vars()?)?),
                }
            }
            ModelType::DistilBert => {
                let config: distilbert::Config = repo.config()?;

                match (
                    model_config.is_sequence_classification(),
                    model_config.is_token_classification(),
                    !config.id2label.is_empty(),
                ) {
                    (true, _, true) => AnyModel::DistilBertSequenceClassifier(distilbert::Classifier::new(
                        repo.vars()?,
                        &config,
                        ClassifyMode::Whole,
                    )?),
                    (_, true, true) => AnyModel::DistilBertEntityClassifier(distilbert::Classifier::new(
                        repo.vars()?,
                        &config,
                        ClassifyMode::Regions,
                    )?),
                    _ => {
                        return Err(Error::custom(
                            "ai::invalid",
                            format!("{name} has no compatible classification head"),
                        ));
                    }
                }
            }
            #[cfg(feature = "vision")]
            ModelType::MobileNetV2 => {
                let config: mobilenet::Config = repo.config()?;
                let processor: mobilenet::Processor = repo.json("preprocessor_config.json")?;
                AnyModel::VisionClassifier(mobilenet::Classifier::new(repo.vars()?, config, processor)?)
            }
            #[cfg(feature = "vision")]
            ModelType::Detr => {
                let config: detr::Config = repo.config()?;
                config.validate()?;
                let processor: detr::Processor = repo.json("preprocessor_config.json")?;
                AnyModel::ObjectDetector(detr::Detector::new(repo.vars()?, config, processor)?)
            }
            #[cfg(feature = "vision")]
            ModelType::Git => {
                let config: git::Config = repo.config()?;
                let processor: git::Processor = repo.json("preprocessor_config.json")?;
                AnyModel::ImageCaptioner(git::Captioner::new(config, processor, repo.vars()?)?)
            }
            #[cfg(feature = "audio")]
            ModelType::Whisper => {
                let config: whisper::Config = repo.config()?;
                let processor: whisper::Processor = repo.json("preprocessor_config.json")?;

                if model_config.is_audio_captioning() {
                    AnyModel::WhisperCaptioner(whisper::Captioner::new(config, processor, repo.vars()?)?)
                } else {
                    AnyModel::Transcriber(whisper::Transcriber::new(config, processor, repo.vars()?)?)
                }
            }
            ModelType::Bert | ModelType::Unknown => {
                let config: bert::Config = repo.config()?;

                match (
                    model_config.is_sequence_classification(),
                    model_config.is_token_classification(),
                    !config.id2label.is_empty(),
                ) {
                    (true, _, true) => {
                        AnyModel::BertSequenceClassifier(bert::Classifier::new(repo.vars()?, &config, ClassifyMode::Whole)?)
                    }
                    (_, true, true) => {
                        AnyModel::BertEntityClassifier(bert::Classifier::new(repo.vars()?, &config, ClassifyMode::Regions)?)
                    }
                    _ => AnyModel::Embedder(keybert::KeyBert::new(bert::Embedder::new(repo.vars()?, &config)?)),
                }
            }
            other => {
                return Err(Error::custom(
                    "ai::invalid",
                    format!("{name} has unsupported architecture `{other}`"),
                ));
            }
        };
        let tokenizer = match &model {
            #[cfg(feature = "audio")]
            AnyModel::AudioClassifier(_) => None,
            #[cfg(feature = "audio")]
            AnyModel::AudioCaptioner(_) => Some(audiocap::tokenizer()?),
            #[cfg(feature = "vision")]
            AnyModel::VisionClassifier(_) | AnyModel::ObjectDetector(_) => None,
            _ => Some(repo.tokenizer()?),
        };

        Ok(Self {
            model,
            tokenizer,
            device,
            name,
            config: Some(model_config),
        })
    }

    pub fn context(&self) -> Context<'_> {
        Context::new(self.tokenizer.as_ref(), &self.device, &self.name)
    }

    pub fn as_embed(&self) -> Option<&dyn Embed> {
        match &self.model {
            AnyModel::Embedder(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub(crate) fn classify_mode(&self, media: ClassifyMedia) -> Result<ClassifyMode, Error> {
        match (media, &self.model) {
            (ClassifyMedia::Text, AnyModel::BartSequenceClassifier(_))
            | (ClassifyMedia::Text, AnyModel::BertSequenceClassifier(_))
            | (ClassifyMedia::Text, AnyModel::DistilBertSequenceClassifier(_)) => Ok(ClassifyMode::Whole),
            (ClassifyMedia::Text, AnyModel::Embedder(_))
            | (ClassifyMedia::Text, AnyModel::BartEntityClassifier(_))
            | (ClassifyMedia::Text, AnyModel::BertEntityClassifier(_))
            | (ClassifyMedia::Text, AnyModel::DistilBertEntityClassifier(_))
            | (ClassifyMedia::Text, AnyModel::BartZeroShotClassifier(_))
            | (ClassifyMedia::Text, AnyModel::Remote(_)) => Ok(ClassifyMode::Regions),
            #[cfg(feature = "vision")]
            (ClassifyMedia::Image, AnyModel::VisionClassifier(_)) => Ok(ClassifyMode::Whole),
            #[cfg(feature = "vision")]
            (ClassifyMedia::Image, AnyModel::ObjectDetector(_)) => Ok(ClassifyMode::Regions),
            #[cfg(feature = "audio")]
            (ClassifyMedia::Audio, AnyModel::AudioClassifier(_)) => Ok(ClassifyMode::Whole),
            _ => Err(Error::custom(
                "ai::invalid",
                format!("{} cannot classify {media:?} input", self.name),
            )),
        }
    }

    pub fn as_classify(&self) -> Option<&dyn Classify> {
        match &self.model {
            AnyModel::Embedder(model) => Some(model),
            AnyModel::BartSequenceClassifier(model) | AnyModel::BartEntityClassifier(model) => Some(model),
            AnyModel::BartZeroShotClassifier(model) => Some(model),
            AnyModel::BertSequenceClassifier(model) | AnyModel::BertEntityClassifier(model) => Some(model),
            AnyModel::DistilBertSequenceClassifier(model) | AnyModel::DistilBertEntityClassifier(model) => Some(model),
            #[cfg(feature = "vision")]
            AnyModel::VisionClassifier(model) => Some(model),
            #[cfg(feature = "vision")]
            AnyModel::ObjectDetector(model) => Some(model),
            #[cfg(feature = "audio")]
            AnyModel::AudioClassifier(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_generate(&self) -> Option<&dyn Generate> {
        match &self.model {
            AnyModel::Summarizer(model) => Some(model),
            #[cfg(feature = "vision")]
            AnyModel::ImageCaptioner(model) => Some(model),
            #[cfg(feature = "audio")]
            AnyModel::Transcriber(model) => Some(model),
            #[cfg(feature = "audio")]
            AnyModel::WhisperCaptioner(model) => Some(model),
            #[cfg(feature = "audio")]
            AnyModel::AudioCaptioner(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn cannot(&self, capability: &str) -> Error {
        Error::custom("ai::invalid", format!("{} cannot {capability}", self.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_config_json_model_type() {
        assert_eq!("bert".parse::<ModelType>().unwrap(), ModelType::Bert);
        assert_eq!("distilbert".parse::<ModelType>().unwrap(), ModelType::DistilBert);
    }

    /// An unrecognised architecture must not be an error -- config.json may name anything.
    #[test]
    fn an_unknown_architecture_is_not_an_error() {
        assert_eq!("mamba".parse::<ModelType>().unwrap(), ModelType::Unknown);
        assert_eq!(serde_json::from_str::<ModelType>("\"mamba\"").unwrap(), ModelType::Unknown);
    }

    #[test]
    fn serde_round_trips() {
        assert_eq!(serde_json::to_string(&ModelType::Bert).unwrap(), "\"bert\"");
        assert_eq!(serde_json::from_str::<ModelType>("\"bart\"").unwrap(), ModelType::Bart);
    }
}
