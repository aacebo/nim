use candle_core::Device;
use tokenizers::Tokenizer;

use crate::models::Batch;

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
