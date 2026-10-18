use candle_core::{IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::model::Whisper;

use crate::Error;

use super::Config;

pub struct Model {
    whisper: Whisper,
}

impl Model {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        Ok(Self {
            whisper: Whisper::load(&vars, config.model().clone())?,
        })
    }

    pub fn encode(&mut self, input: &Tensor) -> Result<Tensor, Error> {
        Ok(self.whisper.encoder.forward(input, true)?)
    }

    pub fn logits(&mut self, audio: &Tensor, tokens: &[u32]) -> Result<Tensor, Error> {
        self.whisper.decoder.reset_kv_cache();

        let input = Tensor::from_slice(tokens, (1, tokens.len()), audio.device())?;
        let output = self.whisper.decoder.forward(&input, audio, true)?;
        let (_, len, _) = output.dims3()?;

        Ok(self.whisper.decoder.final_linear(&output.i((.., len - 1..))?)?.i((0, 0))?)
    }
}
