use candle_core::{Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};

pub struct QwenSampler {
    sampler: LogitsProcessor,
}

impl QwenSampler {
    pub fn new(temperature: f64, top_p: f64) -> Self {
        let sampling = if temperature == 0.0 {
            Sampling::ArgMax
        } else {
            Sampling::TopP { p: top_p, temperature }
        };

        Self {
            sampler: LogitsProcessor::from_sampling(rand::random(), sampling),
        }
    }

    pub fn sample(&mut self, logits: &Tensor) -> nim_error::Result<u32> {
        let logits = logits.to_device(&Device::Cpu)?;
        Ok(self.sampler.sample(&logits)?)
    }
}
