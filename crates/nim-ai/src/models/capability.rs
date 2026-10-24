use candle_core::Device;
use tokenizers::Tokenizer;

use crate::Error;
use crate::pipelines::common::Batch;
use crate::types::Annotation;

/// Text in, text out. BART does this by beam search; a hosted model by its completion endpoint.
pub trait Generate: Send + Sync {
    fn generate(&self, ctx: &Context, text: &[&str], opts: &GenOpts) -> Result<Vec<String>, Error>;
}

/// A sentence vector per input.
pub trait Embed: Send + Sync {
    fn embed(&self, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<f32>>, Error>;
}

pub trait Classify: Send + Sync {
    fn classify(&self, ctx: &Context, text: &[&str], options: &ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error>;
}

/// Image classification
#[cfg(feature = "vision")]
pub trait ImageClassify: Send + Sync {
    fn classify_image(&self, ctx: &Context, image: &[u8], limit: usize) -> Result<Vec<Label>, Error>;
}

#[cfg(feature = "vision")]
pub trait ImageDetect: Send + Sync {
    fn detect_image(&self, ctx: &Context, image: &[u8], min_score: f64, limit: usize) -> Result<Vec<Detection>, Error>;
}

#[cfg(feature = "vision")]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Detection {
    pub class_id: usize,
    pub label: String,
    pub score: f64,
    pub bbox: BoundingBox,
}

#[cfg(feature = "vision")]
#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
}

#[cfg(feature = "audio")]
pub trait AudioClassify: Send + Sync {
    fn classify_audio(&self, ctx: &Context, audio: &[u8], limit: usize) -> Result<Vec<Label>, Error>;
}

#[cfg(feature = "vision")]
pub trait ImageToText: Send + Sync {
    fn image_to_text(
        &self,
        ctx: &Context,
        images: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error>;
}

#[cfg(feature = "audio")]
pub trait Transcribe: Send + Sync {
    fn transcribe(
        &self,
        ctx: &Context,
        audio: &[Vec<u8>],
        language: Option<&str>,
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error>;
}

#[cfg(feature = "audio")]
pub trait AudioToText: Send + Sync {
    fn audio_to_text(
        &self,
        ctx: &Context,
        audio: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error>;
}

/// Borrows what a model needs to turn text into tensors. A loaded model owns its weights; the
/// tokenizer and device are lent per call, so one set of weights can serve several capabilities.
///
/// The tokenizer is optional because a hosted model has none -- it is handed text and returns
/// text. Its capability impls never touch the context, and `tokenizer()` is the error a local
/// model would raise if its tokenizer were somehow missing.
pub struct Context<'a> {
    name: &'a str,
    device: &'a Device,
    tokenizer: Option<&'a Tokenizer>,
}

impl<'a> Context<'a> {
    pub fn new(tokenizer: Option<&'a Tokenizer>, device: &'a Device, name: &'a str) -> Self {
        Self { tokenizer, device, name }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn device(&self) -> &Device {
        self.device
    }

    pub fn tokenizer(&self) -> Result<&Tokenizer, Error> {
        self.tokenizer
            .ok_or_else(|| Error::custom("ai", format!("{} has no tokenizer", self.name)))
    }

    pub fn encode(&self, text: &[&str]) -> Result<Batch, Error> {
        let encodings = self
            .tokenizer()?
            .encode_batch(text.to_vec(), true)
            .map_err(|err| Error::custom("ai::encoding", err))?;
        Batch::new(encodings, self.device)
    }

    pub fn encode_one(&self, text: &str) -> Result<tokenizers::Encoding, Error> {
        self.tokenizer()?
            .encode(text, true)
            .map_err(|err| Error::custom("ai::encode", err))
    }
}

/// One label for a whole input.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub label: String,
    pub score: f64,
}

/// A whole word: sub-word pieces already merged, carrying BYTE offsets into the source text.
#[derive(Debug, Clone)]
pub struct Word {
    pub label: String,
    pub score: f64,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ClassifyMode {
    #[default]
    Text,
    Spans,
}

#[derive(Debug, Clone)]
pub struct ClassifyOptions<'a> {
    pub mode: ClassifyMode,
    pub labels: Option<&'a [String]>,
    pub multi_label: bool,
    pub hypothesis_template: &'a str,
}

impl Default for ClassifyOptions<'_> {
    fn default() -> Self {
        Self {
            mode: ClassifyMode::Text,
            labels: None,
            multi_label: false,
            hypothesis_template: "This example is about {}.",
        }
    }
}

impl ClassifyOptions<'_> {
    pub fn validate(&self) -> Result<(), Error> {
        if self.hypothesis_template.matches("{}").count() != 1 {
            return Err(Error::custom(
                "ai::config",
                "hypothesis_template must contain exactly one {} placeholder",
            ));
        }

        if let Some(labels) = self.labels {
            if labels.is_empty() || labels.iter().any(|label| label.trim().is_empty()) {
                return Err(Error::custom("ai::config", "labels must contain at least one nonblank label"));
            }

            if labels
                .iter()
                .enumerate()
                .any(|(index, label)| labels[..index].contains(label))
            {
                return Err(Error::custom("ai::config", "labels must be unique"));
            }
        }

        Ok(())
    }
}

pub struct GenOpts {
    pub prompt: &'static str,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl Default for GenOpts {
    fn default() -> Self {
        Self {
            prompt: "Summarize the text the user gives you. Be concise and factual; use only \
                     information present in the text.",
            max_len: None,
            beams: None,
        }
    }
}
