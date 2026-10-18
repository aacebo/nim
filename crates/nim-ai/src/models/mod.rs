#[cfg(feature = "audio")]
pub mod audiocap;
pub mod bart;
pub mod bert;
#[cfg(feature = "audio")]
pub mod cnn8rnn;
pub mod distilbert;
#[cfg(feature = "vision")]
pub mod git;
#[cfg(feature = "vision")]
pub mod mobilenet;
#[cfg(feature = "audio")]
pub mod whisper;

mod cache;
mod capability;
mod loader;
mod model_id;
mod provider;
mod remote;
mod tokenizer;

use std::str::FromStr;
use std::sync::Arc;

use candle_core::{DType, Device};

use cache::*;
pub use capability::*;
pub use loader::*;
pub use model_id::*;
pub use provider::*;

use crate::Error;
use crate::clients::fs::FileSystem;
use crate::clients::hf::HuggingFace;
use crate::clients::http::Http;
use crate::clients::openai::OpenAI;
use crate::resources::{Repository, Resource, Uri};

static MODELS: std::sync::LazyLock<Cache<crate::models::Model>> = std::sync::LazyLock::new(Cache::new);

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

/// A model with its weights loaded (or, for a remote model, its endpoint bound). The variant
/// determines which capabilities exist: the `None` arms below are the empty cells of the matrix.
pub enum AnyModel {
    Embedder(bert::Embedder),
    BartEntityClassifier(bart::Classifier),
    BartSequenceClassifier(bart::Classifier),
    BertEntityClassifier(bert::Classifier),
    BertSequenceClassifier(bert::Classifier),
    DistilBertEntityClassifier(distilbert::Classifier),
    DistilBertSequenceClassifier(distilbert::Classifier),
    Summarizer(bart::Summarizer),
    #[cfg(feature = "vision")]
    VisionClassifier(mobilenet::Classifier),
    #[cfg(feature = "vision")]
    ImageToText(git::Captioner),
    #[cfg(feature = "audio")]
    AudioClassifier(cnn8rnn::Classifier),
    #[cfg(feature = "audio")]
    Transcriber(whisper::Transcriber),
    #[cfg(feature = "audio")]
    AudioToText(whisper::Captioner),
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
                Architecture::MobileNetV2 | Architecture::Git => {
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
                        (true, _, false) => AnyModel::BartSequenceClassifier(bart::Classifier::new(&config, repo.vars()?)?),
                        (_, true, false) => AnyModel::BartEntityClassifier(bart::Classifier::new(&config, repo.vars()?)?),
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
                        (true, _, true) => {
                            AnyModel::DistilBertSequenceClassifier(distilbert::Classifier::new(repo.vars()?, &config)?)
                        }
                        (_, true, true) => {
                            AnyModel::DistilBertEntityClassifier(distilbert::Classifier::new(repo.vars()?, &config)?)
                        }
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
                Architecture::Git => {
                    let config: git::Config = repo.config()?;
                    let processor: git::Processor = repo.json("preprocessor_config.json")?;
                    AnyModel::ImageToText(git::Captioner::new(config, processor, repo.vars()?)?)
                }
                #[cfg(feature = "audio")]
                Architecture::Whisper => {
                    let config: whisper::Config = repo.config()?;
                    let processor: whisper::Processor = repo.json("preprocessor_config.json")?;

                    if architecture.is_audio_captioning() {
                        AnyModel::AudioToText(whisper::Captioner::new(config, processor, repo.vars()?)?)
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
                        (true, _, true) => AnyModel::BertSequenceClassifier(bert::Classifier::new(repo.vars()?, &config)?),
                        (_, true, true) => AnyModel::BertEntityClassifier(bert::Classifier::new(repo.vars()?, &config)?),
                        _ => AnyModel::Embedder(bert::Embedder::new(repo.vars()?, &config)?),
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
                    Architecture::MobileNetV2 => None,
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

    pub fn as_sequence_classify(&self) -> Option<&dyn SequenceClassify> {
        match &self.model {
            AnyModel::BartSequenceClassifier(model) => Some(model),
            AnyModel::BertSequenceClassifier(model) => Some(model),
            AnyModel::DistilBertSequenceClassifier(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "vision")]
    pub fn as_image_classify(&self) -> Option<&dyn ImageClassify> {
        match &self.model {
            AnyModel::VisionClassifier(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "vision")]
    pub fn as_image_to_text(&self) -> Option<&dyn ImageToText> {
        match &self.model {
            AnyModel::ImageToText(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "audio")]
    pub fn as_audio_classify(&self) -> Option<&dyn AudioClassify> {
        match &self.model {
            AnyModel::AudioClassifier(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "audio")]
    pub fn as_transcribe(&self) -> Option<&dyn Transcribe> {
        match &self.model {
            AnyModel::Transcriber(model) => Some(model),
            _ => None,
        }
    }

    #[cfg(feature = "audio")]
    pub fn as_audio_to_text(&self) -> Option<&dyn AudioToText> {
        match &self.model {
            AnyModel::AudioToText(model) => Some(model),
            AnyModel::AudioCaptioner(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_entity_classify(&self) -> Option<&dyn EntityClassify> {
        match &self.model {
            AnyModel::BartEntityClassifier(model) => Some(model),
            AnyModel::BertEntityClassifier(model) => Some(model),
            AnyModel::DistilBertEntityClassifier(model) => Some(model),
            AnyModel::Remote(model) => Some(model),
            _ => None,
        }
    }

    pub fn as_generate(&self) -> Option<&dyn Generate> {
        match &self.model {
            AnyModel::Summarizer(model) => Some(model),
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

/// A model whose weights we load and run ourselves. Every variant has weights, so `repository`
/// and `loader` are total -- a model with no weights cannot be built as a `LocalModel`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalModel {
    Hub(ModelId),
    Path(Resource),
}

impl LocalModel {
    pub fn loader(&self, device: Device, dtype: DType) -> Result<Loader, Error> {
        Ok(Loader::new(self.repository()?, device, dtype))
    }

    pub fn repository(&self) -> Result<Arc<dyn Repository>, Error> {
        match self {
            Self::Hub(id) => Ok(Arc::new(HuggingFace::new(id)?)),
            Self::Path(resource) => match &resource.uri {
                Uri::Local(path) => Ok(Arc::new(FileSystem::new(path))),
                Uri::Http(_) => Ok(Arc::new(Http::new(resource.uri.clone()))),
            },
        }
    }
}

impl std::fmt::Display for LocalModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hub(id) => write!(f, "{id}"),
            Self::Path(resource) => write!(f, "{resource}"),
        }
    }
}

/// A model we call over the wire. It has an endpoint, never weights.
#[derive(Debug, Clone)]
pub struct RemoteModel {
    pub provider: Provider,
    pub id: ModelId,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl RemoteModel {
    pub fn new(provider: Provider, id: ModelId) -> Self {
        Self {
            provider,
            id,
            base_url: None,
            api_key: None,
        }
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        self.base_url = url;
        self
    }

    pub fn api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn client(&self) -> OpenAI {
        match self.provider {
            Provider::OpenAI => OpenAI::new(self.id.clone(), self.base_url.clone(), self.api_key.clone()),
        }
    }
}

impl PartialEq for RemoteModel {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider && self.id == other.id && self.base_url == other.base_url
    }
}

impl Eq for RemoteModel {}

impl std::hash::Hash for RemoteModel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.provider.hash(state);
        self.id.hash(state);
        self.base_url.hash(state);
    }
}

impl std::fmt::Display for RemoteModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.provider, self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelRef {
    Local(LocalModel),
    Remote(RemoteModel),
}

impl ModelRef {
    pub fn hub(id: ModelId) -> Self {
        Self::Local(LocalModel::Hub(id))
    }

    pub fn local(uri: Uri) -> Self {
        Self::Local(LocalModel::Path(Resource::new(uri)))
    }

    pub fn remote(provider: Provider, id: ModelId) -> Self {
        Self::Remote(RemoteModel::new(provider, id))
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        if let Self::Remote(remote) = self {
            self = Self::Remote(remote.base_url(url));
        }

        self
    }

    /// The weights-bearing model, or an error naming the remote one that has none.
    pub fn local_or_err(&self) -> Result<&LocalModel, Error> {
        match self {
            Self::Local(model) => Ok(model),
            Self::Remote(remote) => Err(Error::custom(
                "ai::invalid",
                format!("{remote} is a remote model and has no weights"),
            )),
        }
    }
}

impl FromStr for ModelRef {
    type Err = Error;

    fn from_str(model: &str) -> std::result::Result<Self, Self::Err> {
        let scheme = model.starts_with("file://") || model.starts_with("http://") || model.starts_with("https://");

        if scheme || std::path::Path::new(model).is_dir() {
            Ok(ModelRef::local(model.parse::<Uri>()?))
        } else {
            Ok(ModelRef::hub(model.parse::<ModelId>()?))
        }
    }
}

impl std::fmt::Display for ModelRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(model) => write!(f, "{model}"),
            Self::Remote(remote) => write!(f, "{remote}"),
        }
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
