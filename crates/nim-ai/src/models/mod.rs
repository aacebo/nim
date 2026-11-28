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
#[cfg(feature = "vision")]
pub mod mobilenet;
#[cfg(feature = "audio")]
pub mod parler;
pub mod qwen;
pub mod qwen2;
pub mod qwen3;
#[cfg(feature = "audio")]
pub mod whisper;

mod architecture;
#[cfg(feature = "audio")]
mod audio_captioner;
#[cfg(feature = "audio")]
mod audio_classifier;
#[cfg(feature = "audio")]
mod audio_transcriber;
mod batch;
mod cache;
#[cfg(feature = "vision")]
mod image_captioner;
#[cfg(feature = "vision")]
mod image_classifier;
mod loader;
mod model_config;
mod model_id;
mod model_ref;
mod model_type;
mod provider;
mod remote;
#[cfg(feature = "audio")]
mod speech_synthesizer;
mod text_classifier;
mod text_completer;
mod text_embedder;
mod text_splitter;
mod text_summarizer;
mod tokenizer;

pub use architecture::*;
#[cfg(feature = "audio")]
pub use audio_captioner::AudioCaptioner;
#[cfg(feature = "audio")]
pub use audio_classifier::AudioClassifier;
#[cfg(feature = "audio")]
pub use audio_transcriber::AudioTranscriber;
pub use batch::Batch;
use cache::*;
#[cfg(feature = "vision")]
pub use image_captioner::ImageCaptioner;
#[cfg(feature = "vision")]
pub use image_classifier::ImageClassifier;
pub use loader::*;
pub use model_config::*;
pub use model_id::*;
pub use model_ref::*;
pub use model_type::*;
pub use provider::*;
#[cfg(feature = "audio")]
pub use speech_synthesizer::SpeechSynthesizer;
pub use text_classifier::TextClassifier;
pub use text_completer::TextCompleter;
pub use text_embedder::TextEmbedder;
pub use text_splitter::TextSplitter;
pub use text_summarizer::TextSummarizer;

use std::sync::Arc;

use candle_core::{DType, Device};

use crate::resources::Repository;
use crate::{Caption, Classify, ClassifyMedia, ClassifyMode, Complete, Context, Embed, Split, Summarize};
#[cfg(feature = "audio")]
use crate::{Synthesize, Transcribe};

pub(crate) const DEFAULT_MAX_TOKENS: usize = 1024;
static MODELS: std::sync::LazyLock<Cache<crate::models::Model>> = std::sync::LazyLock::new(Cache::new);

pub trait Forward: Send + Sync {
    type Input;
    type Output;

    fn forward(&self, input: Self::Input) -> nim_error::Result<Self::Output>;
}

pub fn id(value: impl AsRef<str>) -> nim_error::Result<ModelId> {
    value.as_ref().parse::<ModelId>()
}

pub fn refer(value: impl AsRef<str>) -> nim_error::Result<ModelRef> {
    value.as_ref().parse::<ModelRef>()
}

pub fn device() -> nim_error::Result<Device> {
    #[cfg(target_os = "macos")]
    {
        Ok(Device::new_metal(0)?)
    }
    #[cfg(not(target_os = "macos"))]
    Ok(Device::Cpu)
}

/// One cache of loaded models, keyed by `(model, api_key)` -- not one per capability. A model used
/// for two routines now loads its weights once.
#[tracing::instrument]
pub async fn load(model: &ModelRef) -> nim_error::Result<Arc<crate::models::Model>> {
    MODELS
        .get_or_build(Key::new(model), async {
            let device = device()?;

            Ok(Arc::new(Model::new(model, device, DType::F32).await?))
        })
        .await
}

pub enum AnyModel {
    TextEmbedder(TextEmbedder),
    TextClassifier(TextClassifier),
    TextCompleter(TextCompleter),
    TextSummarizer(TextSummarizer),
    TextSplitter(TextSplitter),
    #[cfg(feature = "vision")]
    ImageClassifier(ImageClassifier),
    #[cfg(feature = "vision")]
    ImageCaptioner(ImageCaptioner),
    #[cfg(feature = "audio")]
    AudioClassifier(AudioClassifier),
    #[cfg(feature = "audio")]
    AudioCaptioner(AudioCaptioner),
    #[cfg(feature = "audio")]
    AudioTranscriber(AudioTranscriber),
    #[cfg(feature = "audio")]
    SpeechSynthesizer(SpeechSynthesizer),
    Remote(RemoteModel),
}

/// A loaded model plus the tokenizer and device it needs to see text. Lends a [`Context`] per
/// call, so one set of weights serves every capability the model has.
pub struct Model {
    name: String,
    model: AnyModel,
    classifier: Option<TextClassifier>,
    summarizer: Option<TextSummarizer>,
    device: Device,
    tokenizer: Option<tokenizers::Tokenizer>,
    config: Option<ModelConfig>,
}

impl Model {
    pub async fn new(model: &ModelRef, device: Device, dtype: DType) -> nim_error::Result<Self> {
        let name = model.to_string();

        match model {
            ModelRef::Remote(remote) => Ok(Self {
                name,
                model: AnyModel::Remote(remote.clone()),
                classifier: None,
                summarizer: None,
                device,
                tokenizer: None,
                config: None,
            }),
            ModelRef::Local(local) => Self::load(local, device, dtype, name).await,
        }
    }

    pub fn context(&self) -> Context<'_> {
        Context::new(self.tokenizer.as_ref(), &self.device, &self.name)
    }

    pub fn config(&self) -> Option<&ModelConfig> {
        self.config.as_ref()
    }

    pub fn as_embed(&self) -> Option<&dyn Embed> {
        match &self.model {
            AnyModel::TextEmbedder(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_classify(&self) -> Option<&dyn Classify> {
        match (&self.model, &self.classifier) {
            (_, Some(model)) | (AnyModel::TextClassifier(model), _) => Some(model),
            #[cfg(feature = "vision")]
            (AnyModel::ImageClassifier(model), _) => Some(model),
            #[cfg(feature = "audio")]
            (AnyModel::AudioClassifier(model), _) => Some(model),
            (AnyModel::Remote(model), _) => Some(model),
            _ => None,
        }
    }

    pub fn as_complete(&self) -> Option<&dyn Complete> {
        match &self.model {
            AnyModel::TextCompleter(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_split(&self) -> Option<&dyn Split> {
        match &self.model {
            AnyModel::TextSplitter(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_summarize(&self) -> Option<&dyn Summarize> {
        match (&self.model, &self.summarizer) {
            (_, Some(model)) | (AnyModel::TextSummarizer(model), _) => Some(model),
            (AnyModel::Remote(model), _) => Some(model),
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
            AnyModel::AudioCaptioner(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "audio")]
    pub fn as_synthesize(&self) -> Option<&dyn Synthesize> {
        match &self.model {
            AnyModel::SpeechSynthesizer(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "audio")]
    pub fn as_transcribe(&self) -> Option<&dyn Transcribe> {
        match &self.model {
            AnyModel::AudioTranscriber(model) => Some(model),
            _ => None,
        }
    }

    pub fn cannot(&self, capability: &str) -> nim_error::Diagnostic {
        nim_error::Diagnostic::message(format!("{} cannot {capability}", self.name))
    }

    pub fn classify_mode(&self, media: ClassifyMedia) -> nim_error::Result<ClassifyMode> {
        if let (ClassifyMedia::Text, Some(model)) = (media, &self.classifier) {
            return Ok(model.mode());
        }

        match (media, &self.model) {
            (ClassifyMedia::Text, AnyModel::TextClassifier(model)) => Ok(model.mode()),
            (ClassifyMedia::Text, AnyModel::Remote(_)) => Ok(ClassifyMode::Regions),
            #[cfg(feature = "vision")]
            (ClassifyMedia::Image, AnyModel::ImageClassifier(model)) => Ok(model.mode()),
            #[cfg(feature = "audio")]
            (ClassifyMedia::Audio, AnyModel::AudioClassifier(model)) => Ok(model.mode()),
            _ => Err(nim_error::Diagnostic::message(format!(
                "{} cannot classify {media:?} input",
                self.name
            ))),
        }
    }

    pub async fn load(model: &LocalModel, device: Device, dtype: DType, name: String) -> nim_error::Result<Self> {
        let repo = model.loader(device, dtype)?;
        let device = repo.device().clone();

        if let LocalModel::Hub(id) = model
            && let Some(base) = id.to_string().strip_suffix("-GGUF")
        {
            let companion = LocalModel::Hub(base.parse()?);
            let companion = companion.loader(device.clone(), dtype)?;
            let config: ModelConfig = companion.config().await?;
            let weights = match id.to_string().as_str() {
                qwen2::QUANTIZED_MODEL => qwen2::WEIGHTS.to_string(),
                qwen3::QUANTIZED_MODEL => qwen3::WEIGHTS.to_string(),
                qwen3::COMPLETION_QUANTIZED_MODEL => qwen3::COMPLETION_WEIGHTS.to_string(),
                _ => {
                    let files = crate::clients::hf::HuggingFaceRepository::new(id)?.files().await?;

                    files
                        .into_iter()
                        .filter(|file| file.to_ascii_lowercase().ends_with(".gguf"))
                        .filter(|file| !file.contains("-of-"))
                        .min_by_key(|file| {
                            let name = file.to_ascii_lowercase();

                            if name.ends_with("q4_k_m.gguf") {
                                (0, name)
                            } else if name.ends_with("q8_0.gguf") {
                                (1, name)
                            } else {
                                (2, name)
                            }
                        })
                        .ok_or_else(|| nim_error::Diagnostic::message(format!("{id} has no supported GGUF weights")))?
                }
            };
            let model = TextCompleter::quantized(&repo, &companion, config.model_type, &weights).await?;

            return Ok(Self::loaded(
                name,
                model.into(),
                device,
                Some(companion.tokenizer().await?),
                Some(companion.config().await?),
            ));
        }

        let model_config: ModelConfig = repo.config().await?;

        let model = match model_config.model_type {
            ModelType::Qwen2 => TextCompleter::load(&repo, ModelType::Qwen2).await?.into(),
            #[cfg(feature = "audio")]
            _ if model_config.is_cnn8rnn_audio_classification() => AudioClassifier::load(&repo).await?.into(),
            #[cfg(feature = "audio")]
            _ if model_config.is_cnn14_audio_captioning() => AudioCaptioner::load(&repo, &model_config).await?.into(),
            #[cfg(feature = "audio")]
            ModelType::ParlerTTS => SpeechSynthesizer::load(&repo, dtype).await?.into(),
            ModelType::Qwen3 => {
                if model_config
                    .architectures
                    .iter()
                    .any(|architecture| matches!(architecture, Architecture::Qwen3ForCausalLM))
                {
                    TextCompleter::load(&repo, ModelType::Qwen3).await?.into()
                } else if model_config
                    .architectures
                    .iter()
                    .any(|architecture| matches!(architecture, Architecture::Qwen3Model))
                {
                    TextEmbedder::load(&repo, ModelType::Qwen3).await?.into()
                } else {
                    return Err(nim_error::Diagnostic::message(format!(
                        "{name} has an unsupported Qwen3 architecture"
                    )));
                }
            }
            ModelType::Gemma3Text => TextEmbedder::load(&repo, ModelType::Gemma3Text).await?.into(),
            ModelType::Bart => {
                let config: bart::BartConfig = repo.config().await?;

                if !config.id2label.is_empty()
                    && (model_config.is_sequence_classification() || model_config.is_token_classification())
                {
                    TextClassifier::load(&repo, &model_config, &name).await?.into()
                } else {
                    TextSummarizer::load(&repo).await?.into()
                }
            }
            ModelType::DistilBert => {
                let config: distilbert::DistilBertConfig = repo.config().await?;

                if model_config.is_token_classification()
                    && config.id2label.get("0").is_some_and(|label| label == "O")
                    && config.id2label.get("1").is_some_and(|label| label == "separator")
                    && config.id2label.len() == 2
                {
                    TextSplitter::load(&repo).await?.into()
                } else {
                    TextClassifier::load(&repo, &model_config, &name).await?.into()
                }
            }
            #[cfg(feature = "vision")]
            ModelType::MobileNetV2 | ModelType::Detr => ImageClassifier::load(&repo, model_config.model_type).await?.into(),
            #[cfg(feature = "vision")]
            ModelType::Git => ImageCaptioner::load(&repo).await?.into(),
            #[cfg(feature = "audio")]
            ModelType::Whisper => {
                if model_config.is_audio_captioning() {
                    AudioCaptioner::load(&repo, &model_config).await?.into()
                } else {
                    AudioTranscriber::load(&repo).await?.into()
                }
            }
            ModelType::Unknown
                if model_config.architectures.iter().any(|architecture| {
                    !matches!(
                        architecture,
                        Architecture::BertModel
                            | Architecture::BertForTokenClassification
                            | Architecture::BertForSequenceClassification
                            | Architecture::Other(_)
                    )
                }) =>
            {
                return Err(nim_error::Diagnostic::message(format!(
                    "{name} has an unsupported architecture"
                )));
            }
            ModelType::Bert | ModelType::Unknown => {
                let config: bert::BertConfig = repo.config().await?;

                if !config.id2label.is_empty()
                    && (model_config.is_sequence_classification() || model_config.is_token_classification())
                {
                    TextClassifier::load(&repo, &model_config, &name).await?.into()
                } else {
                    TextEmbedder::load(&repo, model_config.model_type).await?.into()
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
            AnyModel::AudioClassifier(AudioClassifier::Cnn8RnnClassifier(_)) => None,
            #[cfg(feature = "audio")]
            AnyModel::AudioCaptioner(AudioCaptioner::AudioCapCaptioner(_)) => Some(audiocap::tokenizer().await?),
            #[cfg(feature = "vision")]
            AnyModel::ImageClassifier(ImageClassifier::MobileNetClassifier(_) | ImageClassifier::DetrDetector(_)) => None,
            _ => Some(repo.tokenizer().await?),
        };

        Ok(Self::loaded(name, model, device, tokenizer, Some(model_config)))
    }

    fn loaded(
        name: String,
        model: AnyModel,
        device: Device,
        tokenizer: Option<tokenizers::Tokenizer>,
        config: Option<ModelConfig>,
    ) -> Self {
        let classifier = match &model {
            AnyModel::TextEmbedder(TextEmbedder::BertEmbedder(model)) => Some(TextClassifier::BertEmbedder(Arc::clone(model))),
            _ => None,
        };

        let summarizer = match &model {
            AnyModel::TextCompleter(TextCompleter::Qwen2Generator(model)) => {
                Some(TextSummarizer::Qwen2Generator(Arc::clone(model)))
            }
            AnyModel::TextCompleter(TextCompleter::Qwen3Generator(model)) => {
                Some(TextSummarizer::Qwen3Generator(Arc::clone(model)))
            }
            _ => None,
        };

        Self {
            name,
            model,
            classifier,
            summarizer,
            device,
            tokenizer,
            config,
        }
    }
}

fn output_limit(model: &str, context: usize, prompt: usize, requested: Option<usize>) -> nim_error::Result<usize> {
    let remaining = context
        .checked_sub(prompt)
        .filter(|remaining| *remaining > 0)
        .ok_or_else(|| nim_error::Diagnostic::message(format!("{model} prompt exceeds the available context")))?;
    Ok(requested.unwrap_or(DEFAULT_MAX_TOKENS).min(remaining))
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
