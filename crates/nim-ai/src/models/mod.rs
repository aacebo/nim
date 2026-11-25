#[cfg(feature = "audio")]
pub mod audiocap;
pub mod bart;
pub mod bert;
#[cfg(feature = "audio")]
pub mod cnn8rnn;
#[cfg(feature = "vision")]
pub mod detr;
pub mod distilbert;
pub mod embeddinggemma;
#[cfg(feature = "vision")]
pub mod git;
pub mod keybert;
#[cfg(feature = "vision")]
pub mod mobilenet;
pub mod qwen2;
pub mod qwen3;
#[cfg(feature = "audio")]
pub mod whisper;

mod architecture;
mod batch;
mod cache;
mod loader;
mod model_config;
mod model_id;
mod model_ref;
mod model_type;
mod provider;
mod remote;
mod tokenizer;

pub use architecture::*;
pub use batch::Batch;
use cache::*;
pub use loader::*;
pub use model_config::*;
pub use model_id::*;
pub use model_ref::*;
pub use model_type::*;
pub use provider::*;

use candle_core::{DType, Device};

use crate::resources::Repository;
pub(crate) use crate::{
    Caption, CaptionOptions, Chat, Classify, ClassifyMedia, ClassifyMode, ClassifyOptions, Complete, Context, Embed, EmbedMode,
    EmbedOptions, Summarize, SummarizeOptions, Transcribe, TranscribeOptions,
};

pub(crate) const DEFAULT_MAX_TOKENS: usize = 1024;
static MODELS: std::sync::LazyLock<Cache<crate::models::Model>> = std::sync::LazyLock::new(Cache::new);

/// A model with its weights loaded (or, for a remote model, its endpoint bound). The variant
/// determines which capabilities exist: the `None` arms below are the empty cells of the matrix.
pub enum AnyModel {
    Embedder(keybert::KeyBert<bert::BertEmbedder>),
    BartEntityClassifier(bart::BartClassifier),
    BartSequenceClassifier(bart::BartClassifier),
    BartZeroShotClassifier(bart::BartZeroShotClassifier),
    BertEntityClassifier(bert::BertClassifier),
    BertSequenceClassifier(bert::BertClassifier),
    DistilBertEntityClassifier(distilbert::DistilBertClassifier),
    DistilBertSequenceClassifier(distilbert::DistilBertClassifier),
    Summarizer(bart::BartSummarizer),
    QwenGenerator(qwen2::Qwen2Generator),
    QwenEmbedder(qwen3::Qwen3Embedder),
    Qwen3Generator(qwen3::Qwen3Generator),
    GemmaEmbedder(embeddinggemma::EmbeddingGemmaEmbedder),
    #[cfg(feature = "vision")]
    VisionClassifier(mobilenet::MobileNetClassifier),
    #[cfg(feature = "vision")]
    ObjectDetector(detr::DetrDetector),
    #[cfg(feature = "vision")]
    ImageCaptioner(git::GitCaptioner),
    #[cfg(feature = "audio")]
    AudioClassifier(cnn8rnn::Cnn8RnnClassifier),
    #[cfg(feature = "audio")]
    Transcriber(whisper::WhisperTranscriber),
    #[cfg(feature = "audio")]
    WhisperCaptioner(whisper::WhisperCaptioner),
    #[cfg(feature = "audio")]
    AudioCaptioner(audiocap::AudioCapCaptioner),
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

pub trait Forward: Send + Sync {
    type Input;
    type Output;

    fn forward(&self, input: Self::Input) -> nim_error::Result<Self::Output>;
}

impl Model {
    pub async fn new(model: &ModelRef, device: Device, dtype: DType) -> nim_error::Result<Self> {
        let name = model.to_string();

        match model {
            ModelRef::Remote(remote) => Ok(Self {
                name,
                model: AnyModel::Remote(remote.clone()),
                device,
                tokenizer: None,
                config: None,
            }),
            ModelRef::Local(local) => Self::local(local, device, dtype, name).await,
        }
    }

    pub async fn local(model: &LocalModel, device: Device, dtype: DType, name: String) -> nim_error::Result<Self> {
        let repo = model.loader(device, dtype)?;
        let device = repo.device().clone();

        if matches!(model, LocalModel::Hub(id) if id.to_string() == qwen2::QUANTIZED_MODEL) {
            let companion = LocalModel::Hub(qwen2::MODEL.parse()?);
            let companion = companion.loader(device.clone(), dtype)?;
            let config: qwen2::Config = companion.config().await?;
            let weights = repo.resolve(std::path::Path::new(qwen2::WEIGHTS)).await?;

            return Ok(Self {
                name,
                model: AnyModel::QwenGenerator(qwen2::Qwen2Generator::quantized(&config, weights, &device)?),
                device,
                tokenizer: Some(companion.tokenizer().await?),
                config: Some(companion.config().await?),
            });
        }

        let qwen3_completion = matches!(model, LocalModel::Hub(id) if id.to_string() == qwen3::COMPLETION_QUANTIZED_MODEL);
        let qwen3_quantized = matches!(model, LocalModel::Hub(id) if {
            let id = id.to_string();
            id == qwen3::QUANTIZED_MODEL || id == qwen3::COMPLETION_QUANTIZED_MODEL
        });

        if qwen3_quantized {
            let companion = LocalModel::Hub(
                if qwen3_completion {
                    qwen3::COMPLETION_MODEL
                } else {
                    qwen3::MODEL
                }
                .parse()?,
            );
            let companion = companion.loader(device.clone(), dtype)?;
            let config: qwen3::Config = companion.config().await?;
            let weights = repo
                .resolve(std::path::Path::new(if qwen3_completion {
                    qwen3::COMPLETION_WEIGHTS
                } else {
                    qwen3::WEIGHTS
                }))
                .await?;

            return Ok(Self {
                name,
                model: AnyModel::Qwen3Generator(qwen3::Qwen3Generator::quantized(&config, weights, &device)?),
                device,
                tokenizer: Some(companion.tokenizer().await?),
                config: Some(companion.config().await?),
            });
        }

        let model_config: ModelConfig = repo.config().await?;
        let model = match model_config.model_type {
            ModelType::Qwen2 => {
                let config: qwen2::Config = repo.config().await?;
                AnyModel::QwenGenerator(qwen2::Qwen2Generator::new(&config, repo.vars().await?)?)
            }
            #[cfg(feature = "audio")]
            _ if model_config.is_cnn8rnn_audio_classification() => {
                let config: cnn8rnn::Cnn8RnnConfig = repo.config().await?;
                let labels = repo.string("classes.txt").await?.lines().map(str::to_string).collect();
                AnyModel::AudioClassifier(cnn8rnn::Cnn8RnnClassifier::new(config, labels, repo.vars().await?)?)
            }
            #[cfg(feature = "audio")]
            _ if model_config.is_cnn14_audio_captioning() => {
                let config: audiocap::AudioCapConfig = repo.config().await?;
                AnyModel::AudioCaptioner(audiocap::AudioCapCaptioner::new(config, repo.vars().await?)?)
            }
            #[cfg(not(feature = "audio"))]
            _ if model_config.architectures.iter().any(|architecture| {
                matches!(
                    architecture,
                    Architecture::Cnn8RnnSoundEventDetection | Architecture::Cnn14RnnTempAttnGruModel
                )
            }) =>
            {
                return Err(nim_error::Diagnostic::message(format!("{name} requires the `audio` feature")));
            }
            #[cfg(not(feature = "vision"))]
            ModelType::MobileNetV2 | ModelType::Git | ModelType::Detr => {
                return Err(nim_error::Diagnostic::message(format!(
                    "{name} requires the `vision` feature"
                )));
            }
            #[cfg(not(feature = "audio"))]
            ModelType::Whisper => {
                return Err(nim_error::Diagnostic::message(format!("{name} requires the `audio` feature")));
            }
            ModelType::Qwen3 => {
                if model_config
                    .architectures
                    .iter()
                    .any(|architecture| matches!(architecture, Architecture::Qwen3ForCausalLM))
                {
                    let config: qwen3::Config = repo.config().await?;
                    AnyModel::Qwen3Generator(qwen3::Qwen3Generator::new(&config, repo.vars().await?)?)
                } else if model_config
                    .architectures
                    .iter()
                    .any(|architecture| matches!(architecture, Architecture::Qwen3Model))
                {
                    AnyModel::QwenEmbedder(qwen3::Qwen3Embedder::new(&repo).await?)
                } else {
                    return Err(nim_error::Diagnostic::message(format!(
                        "{name} has an unsupported Qwen3 architecture"
                    )));
                }
            }
            ModelType::Gemma3Text => AnyModel::GemmaEmbedder(embeddinggemma::EmbeddingGemmaEmbedder::new(&repo).await?),
            ModelType::Bart => {
                let config: bart::BartConfig = repo.config().await?;

                match (
                    model_config.is_sequence_classification(),
                    model_config.is_token_classification(),
                    config.id2label.is_empty(),
                ) {
                    (true, _, false) if bart::BartZeroShotClassifier::supports(&config) => {
                        AnyModel::BartZeroShotClassifier(bart::BartZeroShotClassifier::new(&config, repo.vars().await?)?)
                    }
                    (true, _, false) => AnyModel::BartSequenceClassifier(bart::BartClassifier::new(
                        &config,
                        repo.vars().await?,
                        ClassifyMode::Whole,
                    )?),
                    (_, true, false) => AnyModel::BartEntityClassifier(bart::BartClassifier::new(
                        &config,
                        repo.vars().await?,
                        ClassifyMode::Regions,
                    )?),
                    _ => AnyModel::Summarizer(bart::BartSummarizer::new(&config, repo.vars().await?)?),
                }
            }
            ModelType::DistilBert => {
                let config: distilbert::DistilBertConfig = repo.config().await?;

                match (
                    model_config.is_sequence_classification(),
                    model_config.is_token_classification(),
                    !config.id2label.is_empty(),
                ) {
                    (true, _, true) => AnyModel::DistilBertSequenceClassifier(distilbert::DistilBertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Whole,
                    )?),
                    (_, true, true) => AnyModel::DistilBertEntityClassifier(distilbert::DistilBertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Regions,
                    )?),
                    _ => {
                        return Err(nim_error::Diagnostic::message(format!(
                            "{name} has no compatible classification head"
                        )));
                    }
                }
            }
            #[cfg(feature = "vision")]
            ModelType::MobileNetV2 => {
                let config: mobilenet::MobileNetConfig = repo.config().await?;
                let processor: mobilenet::MobileNetProcessor = repo.json("preprocessor_config.json").await?;
                AnyModel::VisionClassifier(mobilenet::MobileNetClassifier::new(repo.vars().await?, config, processor)?)
            }
            #[cfg(feature = "vision")]
            ModelType::Detr => {
                let config: detr::DetrConfig = repo.config().await?;
                config.validate()?;
                let processor: detr::DetrProcessor = repo.json("preprocessor_config.json").await?;
                AnyModel::ObjectDetector(detr::DetrDetector::new(repo.vars().await?, config, processor)?)
            }
            #[cfg(feature = "vision")]
            ModelType::Git => {
                let config: git::GitConfig = repo.config().await?;
                let processor: git::GitProcessor = repo.json("preprocessor_config.json").await?;
                AnyModel::ImageCaptioner(git::GitCaptioner::new(config, processor, repo.vars().await?)?)
            }
            #[cfg(feature = "audio")]
            ModelType::Whisper => {
                let config: whisper::WhisperConfig = repo.config().await?;
                let processor: whisper::WhisperProcessor = repo.json("preprocessor_config.json").await?;

                if model_config.is_audio_captioning() {
                    AnyModel::WhisperCaptioner(whisper::WhisperCaptioner::new(config, processor, repo.vars().await?)?)
                } else {
                    AnyModel::Transcriber(whisper::WhisperTranscriber::new(config, processor, repo.vars().await?)?)
                }
            }
            ModelType::Bert | ModelType::Unknown => {
                let config: bert::BertConfig = repo.config().await?;

                match (
                    model_config.is_sequence_classification(),
                    model_config.is_token_classification(),
                    !config.id2label.is_empty(),
                ) {
                    (true, _, true) => AnyModel::BertSequenceClassifier(bert::BertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Whole,
                    )?),
                    (_, true, true) => AnyModel::BertEntityClassifier(bert::BertClassifier::new(
                        repo.vars().await?,
                        &config,
                        ClassifyMode::Regions,
                    )?),
                    _ => AnyModel::Embedder(keybert::KeyBert::new(bert::BertEmbedder::new(repo.vars().await?, &config)?)),
                }
            }
            other => {
                return Err(nim_error::Diagnostic::message(format!(
                    "{name} has unsupported architecture `{other}`"
                )));
            }
        };
        let tokenizer = match &model {
            #[cfg(feature = "audio")]
            AnyModel::AudioClassifier(_) => None,
            #[cfg(feature = "audio")]
            AnyModel::AudioCaptioner(_) => Some(audiocap::tokenizer().await?),
            #[cfg(feature = "vision")]
            AnyModel::VisionClassifier(_) | AnyModel::ObjectDetector(_) => None,
            _ => Some(repo.tokenizer().await?),
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

    pub fn config(&self) -> Option<&ModelConfig> {
        self.config.as_ref()
    }

    pub fn as_embed(&self) -> Option<&dyn Embed> {
        match &self.model {
            AnyModel::QwenEmbedder(model) => Some(model),
            AnyModel::GemmaEmbedder(model) => Some(model),
            AnyModel::Embedder(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
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

    pub fn as_complete(&self) -> Option<&dyn Complete> {
        match &self.model {
            AnyModel::QwenGenerator(model) => Some(model),
            AnyModel::Qwen3Generator(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_summarize(&self) -> Option<&dyn Summarize> {
        match &self.model {
            AnyModel::Summarizer(model) => Some(model),
            AnyModel::QwenGenerator(model) => Some(model),
            AnyModel::Qwen3Generator(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_image_caption(&self) -> Option<&dyn Caption> {
        match &self.model {
            #[cfg(feature = "vision")]
            AnyModel::ImageCaptioner(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_audio_caption(&self) -> Option<&dyn Caption> {
        match &self.model {
            #[cfg(feature = "audio")]
            AnyModel::WhisperCaptioner(model) => Some(model),
            #[cfg(feature = "audio")]
            AnyModel::AudioCaptioner(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_transcribe(&self) -> Option<&dyn Transcribe> {
        match &self.model {
            #[cfg(feature = "audio")]
            AnyModel::Transcriber(model) => Some(model),
            _ => None,
        }
    }

    pub fn cannot(&self, capability: &str) -> nim_error::Diagnostic {
        nim_error::Diagnostic::message(format!("{} cannot {capability}", self.name))
    }

    pub(crate) fn classify_mode(&self, media: ClassifyMedia) -> nim_error::Result<ClassifyMode> {
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
            _ => Err(nim_error::Diagnostic::message(format!(
                "{} cannot classify {media:?} input",
                self.name
            ))),
        }
    }
}

pub fn id(value: impl AsRef<str>) -> nim_error::Result<ModelId> {
    value.as_ref().parse::<ModelId>()
}

pub fn refer(value: impl AsRef<str>) -> nim_error::Result<ModelRef> {
    value.as_ref().parse::<ModelRef>()
}

/// One cache of loaded models, keyed by `(model, api_key)` -- not one per capability. A model used
/// for two routines now loads its weights once.
#[tracing::instrument]
pub async fn load(model: &ModelRef) -> nim_error::Result<std::sync::Arc<crate::models::Model>> {
    use candle_core::DType;

    MODELS
        .get_or_build(Key::new(model), async {
            let device = device()?;

            Ok(std::sync::Arc::new(Model::new(model, device, DType::F32).await?))
        })
        .await
}

fn output_limit(model: &str, context: usize, prompt: usize, requested: Option<usize>) -> nim_error::Result<usize> {
    let remaining = context
        .checked_sub(prompt)
        .filter(|remaining| *remaining > 0)
        .ok_or_else(|| nim_error::Diagnostic::message(format!("{model} prompt exceeds the available context")))?;
    Ok(requested.unwrap_or(DEFAULT_MAX_TOKENS).min(remaining))
}

fn device() -> nim_error::Result<Device> {
    #[cfg(target_os = "macos")]
    {
        Ok(Device::new_metal(0)?)
    }
    #[cfg(not(target_os = "macos"))]
    Ok(Device::Cpu)
}

#[cfg(test)]
mod tests {
    #[test]
    fn output_limit_uses_remaining_context() {
        assert_eq!(super::output_limit("model", 32768, 100, None).unwrap(), 1024);
        assert_eq!(super::output_limit("model", 32768, 100, Some(512)).unwrap(), 512);
        assert_eq!(super::output_limit("model", 100, 4, Some(1)).unwrap(), 1);
        assert_eq!(super::output_limit("model", 100, 4, None).unwrap(), 96);
        assert!(super::output_limit("model", 32768, 32768, None).is_err());
        assert_eq!(super::output_limit("model", 32768, 100, Some(32669)).unwrap(), 32668);
    }

    use super::*;

    #[test]
    fn parses_a_config_json_model_type() {
        assert_eq!("bert".parse::<ModelType>().unwrap(), ModelType::Bert);
        assert_eq!("distilbert".parse::<ModelType>().unwrap(), ModelType::DistilBert);
    }

    #[test]
    fn parses_qwen3_causal_architecture() {
        assert_eq!(
            "Qwen3ForCausalLM".parse::<Architecture>().unwrap(),
            Architecture::Qwen3ForCausalLM
        );
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
