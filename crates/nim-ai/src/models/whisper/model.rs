use candle_core::{IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::model::Whisper;

use super::WhisperConfig;

pub struct WhisperModel {
    whisper: Whisper,
}

impl WhisperModel {
    pub fn new(config: &WhisperConfig, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            whisper: Whisper::load(&vars, config.model().clone())?,
        })
    }

    pub fn encode(&mut self, input: &Tensor) -> nim_error::Result<Tensor> {
        Ok(self.whisper.encoder.forward(input, true)?)
    }

    pub fn logits(&mut self, audio: &Tensor, tokens: &[u32]) -> nim_error::Result<Tensor> {
        self.whisper.decoder.reset_kv_cache();

        let input = Tensor::from_slice(tokens, (1, tokens.len()), audio.device())?;
        let output = self.whisper.decoder.forward(&input, audio, true)?;
        let (_, len, _) = output.dims3()?;

        Ok(self.whisper.decoder.final_linear(&output.i((.., len - 1..))?)?.i((0, 0))?)
    }
}
