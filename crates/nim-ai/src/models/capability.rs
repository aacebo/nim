use candle_core::Device;
use tokenizers::Tokenizer;

use crate::Error;
use crate::pipelines::common::Batch;
use crate::types::Entity;

/// Text in, text out. BART does this by beam search; a hosted model by its completion endpoint.
pub trait Generate: Send + Sync {
    fn generate(&self, cx: &Context, text: &[&str], opts: &GenOpts) -> Result<Vec<String>, Error>;
}

/// A sentence vector per input.
pub trait Embed: Send + Sync {
    fn embed(&self, cx: &Context, text: &[&str]) -> Result<Vec<Vec<f32>>, Error>;
}

/// Sequence classification: one label per input.
pub trait SequenceClassify: Send + Sync {
    fn classify_sequence(&self, cx: &Context, text: &[&str]) -> Result<Vec<Label>, Error>;
}

/// Image classification
pub trait ImageClassify: Send + Sync {
    fn classify_image(&self, cx: &Context, image: &[u8], limit: usize) -> Result<Vec<Label>, Error>;
}

pub trait ImageToText: Send + Sync {
    fn image_to_text(
        &self,
        cx: &Context,
        images: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error>;
}

pub trait Transcribe: Send + Sync {
    fn transcribe(
        &self,
        cx: &Context,
        audio: &[Vec<u8>],
        language: Option<&str>,
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error>;
}

pub trait AudioToText: Send + Sync {
    fn audio_to_text(
        &self,
        cx: &Context,
        audio: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error>;
}

pub trait EntityClassify: Send + Sync {
    fn classify_entities(&self, cx: &Context, text: &[&str]) -> Result<Vec<Vec<Entity>>, Error>;
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
