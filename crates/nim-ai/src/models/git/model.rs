use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, VarBuilder, linear};

use crate::Error;

use super::config::Config;
use super::decoder::Decoder;
use super::vision::{Projection, Vision};

pub struct Model {
    vision: Vision,
    projection: Projection,
    text: Decoder,
    output: Linear,
}

impl Model {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        let git = vars.pp("git");

        Ok(Self {
            vision: Vision::new(&config.vision_config, git.pp("image_encoder").pp("vision_model"))?,
            projection: Projection::new(
                config.hidden_size,
                config.vision_config.layer_norm_eps,
                git.pp("visual_projection").pp("visual_projection"),
            )?,
            text: Decoder::new(config, git)?,
            output: linear(config.hidden_size, config.vocab_size, vars.pp("output"))?,
        })
    }

    pub fn vision(&mut self, image: &Tensor) -> Result<Tensor, Error> {
        self.vision
            .forward(image)
            .map_err(Into::into)
            .and_then(|xs| self.projection.forward(&xs).map_err(Into::into))
    }

    pub fn logits(&mut self, visual: &Tensor, tokens: &[u32]) -> Result<Tensor, Error> {
        self.text
            .forward(visual, tokens)
            .and_then(|xs| xs.apply(&self.output).map_err(Into::into))
            .and_then(|xs| xs.i((.., xs.dim(1)? - 1, ..)).map_err(Into::into))
    }
}
