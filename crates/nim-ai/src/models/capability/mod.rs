mod caption;
mod chat;
mod classify;
mod embed;
mod summarize;
mod transcribe;

pub use caption::*;
pub use chat::*;
pub use classify::*;
pub use embed::*;
pub use summarize::*;
pub use transcribe::*;

use candle_core::Device;
use tokenizers::Tokenizer;

use super::Batch;

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

    pub fn tokenizer(&self) -> nim_error::Result<&Tokenizer> {
        self.tokenizer
            .ok_or_else(|| nim_error::Diagnostic::message(format!("{} has no tokenizer", self.name)))
    }

    pub fn encode(&self, text: &[&str]) -> nim_error::Result<Batch> {
        let encodings = self
            .tokenizer()?
            .encode_batch(text.to_vec(), true)
            .map_err(nim_error::Diagnostic::message)?;
        Batch::new(encodings, self.device)
    }

    pub fn encode_one(&self, text: &str) -> nim_error::Result<tokenizers::Encoding> {
        self.tokenizer()?.encode(text, true).map_err(nim_error::Diagnostic::message)
    }
}
