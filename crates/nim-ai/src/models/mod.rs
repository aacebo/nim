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

mod cache;
mod capability;
mod loader;
mod model_id;
mod model_ref;
mod provider;
mod remote;
mod tokenizer;

use cache::*;
pub use capability::*;
pub use loader::*;
pub use model_id::*;
pub use model_ref::*;
pub use provider::*;

use candle_core::{DType, Device};
use std::str::FromStr;

use crate::Error;
use crate::resources::Repository;

static MODELS: std::sync::LazyLock<Cache<crate::models::Model>> = std::sync::LazyLock::new(Cache::new);

pub fn id(value: impl AsRef<str>) -> Result<ModelId, Error> {
    value.as_ref().parse::<ModelId>()
}

pub fn refer(value: impl AsRef<str>) -> Result<ModelRef, Error> {
    value.as_ref().parse::<ModelRef>()
}

pub trait Forward: Send + Sync {
    type Input;
    type Output;

    fn forward(&self, input: Self::Input) -> Result<Self::Output, Error>;
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelArgs {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl ModelArgs {
    pub fn resolve(self, default: ModelRef) -> Result<ModelRef, Error> {
        let Some(provider) = self.provider else {
            let Some(model) = self.model else {
                return Ok(default);
            };

            return model.parse();
        };

        let provider: Provider = provider.parse()?;
        let id: ModelId = self
            .model
            .ok_or(Error::custom("ai::config", "model is required when provider is set"))?
            .parse()?;

        Ok(ModelRef::remote(provider, id).base_url(self.base_url))
    }
}

/// One cache of loaded models, keyed by `(model, api_key)` -- not one per capability. A model used
/// for two routines now loads its weights once.
#[tracing::instrument]
pub fn load(model: &ModelRef, api_key: &Option<String>) -> Result<std::sync::Arc<crate::models::Model>, Error> {
    use candle_core::{DType, Device};

    MODELS.get_or_build(Key::new(model, api_key), || {
        Ok(std::sync::Arc::new(crate::models::Model::new(
            model,
            api_key,
            Device::Cpu,
            DType::F32,
        )?))
    })
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    Bart,
    Bert,
    DistilBert,
    Roberta,
    Deberta,
    Gpt2,
    Llama,
    Mistral,
    T5,
    #[serde(rename = "mobilenet_v2")]
    MobileNetV2,
    Detr,
    Git,
    Whisper,
    #[default]
    #[serde(other)]
    Unknown,
}

impl Architecture {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bart => "bart",
            Self::Bert => "bert",
            Self::DistilBert => "distilbert",
            Self::Roberta => "roberta",
            Self::Deberta => "deberta",
            Self::Gpt2 => "gpt2",
            Self::Llama => "llama",
            Self::Mistral => "mistral",
            Self::T5 => "t5",
            Self::MobileNetV2 => "mobilenet_v2",
            Self::Detr => "detr",
            Self::Git => "git",
            Self::Whisper => "whisper",
            Self::Unknown => "??",
        }
    }
}

impl FromStr for Architecture {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Error> {
        Ok(match value.to_lowercase().as_str() {
            "bart" => Self::Bart,
            "bert" => Self::Bert,
            "distilbert" => Self::DistilBert,
            "roberta" => Self::Roberta,
            "deberta" => Self::Deberta,
            "gpt2" => Self::Gpt2,
            "llama" => Self::Llama,
            "mistral" => Self::Mistral,
            "t5" => Self::T5,
            "mobilenet_v2" => Self::MobileNetV2,
            "detr" => Self::Detr,
            "git" => Self::Git,
            "whisper" => Self::Whisper,
            _ => Self::Unknown,
        })
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
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
}

impl Model {
    pub fn new(model: &ModelRef, api_key: &Option<String>, device: Device, dtype: DType) -> Result<Self, Error> {
        let name = model.to_string();

        match model {
            ModelRef::Remote(remote) => Ok(Self {
                name,
                model: AnyModel::Remote(remote.clone().api_key(api_key.clone())),
                device,
                tokenizer: None,
            }),
            ModelRef::Local(local) => Self::local(local, device, dtype, name),
        }
    }

    pub fn local(model: &LocalModel, device: Device, dtype: DType, name: String) -> Result<Self, Error> {
        let repo = model.loader(device, dtype)?;
        let device = repo.device().clone();
        let architecture: Probe = repo.config()?;
        let model = if architecture.is_cnn8rnn_audio_classification() {
            #[cfg(feature = "audio")]
            {
                let config: cnn8rnn::Config = repo.config()?;
                let labels = repo.string("classes.txt")?.lines().map(str::to_string).collect();
                AnyModel::AudioClassifier(cnn8rnn::Classifier::new(config, labels, repo.vars()?)?)
            }
            #[cfg(not(feature = "audio"))]
            return Err(Error::custom("ai::config", format!("{name} requires the `audio` feature")));
        } else if architecture.is_cnn14_audio_captioning() {
            #[cfg(feature = "audio")]
            {
                let config: audiocap::Config = repo.config()?;
                AnyModel::AudioCaptioner(audiocap::Captioner::new(config, repo.vars()?)?)
            }
            #[cfg(not(feature = "audio"))]
            return Err(Error::custom("ai::config", format!("{name} requires the `audio` feature")));
        } else {
            match architecture.architecture {
                #[cfg(not(feature = "vision"))]
                Architecture::MobileNetV2 | Architecture::Git | Architecture::Detr => {
                    return Err(Error::custom("ai::config", format!("{name} requires the `vision` feature")));
                }
                #[cfg(not(feature = "audio"))]
                Architecture::Whisper => {
                    return Err(Error::custom("ai::config", format!("{name} requires the `audio` feature")));
                }
                Architecture::Bart => {
                    let config: bart::Config = repo.config()?;

                    match (
                        architecture.is_sequence_classification(),
                        architecture.is_token_classification(),
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
                Architecture::DistilBert => {
                    let config: distilbert::Config = repo.config()?;

                    match (
                        architecture.is_sequence_classification(),
                        architecture.is_token_classification(),
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
                Architecture::MobileNetV2 => {
                    let config: mobilenet::Config = repo.config()?;
                    let processor: mobilenet::Processor = repo.json("preprocessor_config.json")?;
                    AnyModel::VisionClassifier(mobilenet::Classifier::new(repo.vars()?, config, processor)?)
                }
                #[cfg(feature = "vision")]
                Architecture::Detr => {
                    let config: detr::Config = repo.config()?;
                    config.validate()?;
                    let processor: detr::Processor = repo.json("preprocessor_config.json")?;
                    AnyModel::ObjectDetector(detr::Detector::new(repo.vars()?, config, processor)?)
                }
                #[cfg(feature = "vision")]
                Architecture::Git => {
                    let config: git::Config = repo.config()?;
                    let processor: git::Processor = repo.json("preprocessor_config.json")?;
                    AnyModel::ImageCaptioner(git::Captioner::new(config, processor, repo.vars()?)?)
                }
                #[cfg(feature = "audio")]
                Architecture::Whisper => {
                    let config: whisper::Config = repo.config()?;
                    let processor: whisper::Processor = repo.json("preprocessor_config.json")?;

                    if architecture.is_audio_captioning() {
                        AnyModel::WhisperCaptioner(whisper::Captioner::new(config, processor, repo.vars()?)?)
                    } else {
                        AnyModel::Transcriber(whisper::Transcriber::new(config, processor, repo.vars()?)?)
                    }
                }
                Architecture::Bert | Architecture::Unknown => {
                    let config: bert::Config = repo.config()?;

                    match (
                        architecture.is_sequence_classification(),
                        architecture.is_token_classification(),
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
            }
        };

        Ok(Self {
            model,
            tokenizer: match (
                architecture.is_cnn8rnn_audio_classification(),
                architecture.is_cnn14_audio_captioning(),
            ) {
                (true, _) => None,
                #[cfg(feature = "audio")]
                (_, true) => Some(audiocap::tokenizer()?),
                _ => match architecture.architecture {
                    Architecture::MobileNetV2 | Architecture::Detr => None,
                    _ => Some(repo.tokenizer()?),
                },
            },
            device,
            name,
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

/// Reads just the architecture out of `config.json`, before committing to a concrete config type.
#[derive(serde::Deserialize)]
struct Probe {
    #[serde(default, rename = "model_type")]
    architecture: Architecture,

    #[serde(default)]
    architectures: Vec<String>,
}

impl Probe {
    fn is_token_classification(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.ends_with("ForTokenClassification"))
    }

    fn is_sequence_classification(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.ends_with("ForSequenceClassification"))
    }

    #[cfg(feature = "audio")]
    fn is_audio_captioning(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture == "WhisperForAudioCaptioning")
    }

    fn is_cnn14_audio_captioning(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture == "Cnn14RnnTempAttnGruModel")
    }

    fn is_cnn8rnn_audio_classification(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture == "Cnn8RnnSoundEventDetection")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_config_json_model_type() {
        assert_eq!("bert".parse::<Architecture>().unwrap(), Architecture::Bert);
        assert_eq!("distilbert".parse::<Architecture>().unwrap(), Architecture::DistilBert);
    }

    /// An unrecognised architecture must not be an error -- config.json may name anything.
    #[test]
    fn an_unknown_architecture_is_not_an_error() {
        assert_eq!("mamba".parse::<Architecture>().unwrap(), Architecture::Unknown);
        assert_eq!(
            serde_json::from_str::<Architecture>("\"mamba\"").unwrap(),
            Architecture::Unknown
        );
    }

    #[test]
    fn serde_round_trips() {
        assert_eq!(serde_json::to_string(&Architecture::Bert).unwrap(), "\"bert\"");
        assert_eq!(serde_json::from_str::<Architecture>("\"bart\"").unwrap(), Architecture::Bart);
    }
}
