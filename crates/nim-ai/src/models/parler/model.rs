use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::dac;
use candle_transformers::models::parler_tts::Model;

use super::config::ParlerConfig;
use super::decoder::ParlerDecoder;
use super::weights::ParlerWeights;

pub struct ParlerModel {
    decoder: ParlerDecoder,
    audio_encoder: dac::Model,
}

impl ParlerModel {
    pub fn new(config: &ParlerConfig, vars: VarBuilder<'static>) -> nim_error::Result<Self> {
        let dtype = vars.dtype();
        let device = vars.device().clone();
        let vars = VarBuilder::from_backend(
            Box::new(ParlerWeights::new(vars, config.model.decoder.vocab_size)),
            dtype,
            device,
        );
        let model = Model::new(&config.model, vars)?;
        let audio_encoder = model.audio_encoder.clone();

        Ok(Self {
            decoder: ParlerDecoder::new(config, model),
            audio_encoder,
        })
    }

    pub fn generate(&mut self, prompt: &Tensor, description: &Tensor) -> nim_error::Result<Vec<f32>> {
        let device = prompt.device();
        let codes = self.decoder.forward(prompt, description)?;
        let samples = self.audio_encoder.decode_codes(&codes.to_device(device)?)?;
        Ok(samples.i((0, 0))?.to_device(&Device::Cpu)?.to_dtype(DType::F32)?.to_vec1()?)
    }
}
