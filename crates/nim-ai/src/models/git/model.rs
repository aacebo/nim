use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, VarBuilder, linear};

use super::config::GitConfig;
use super::decoder::GitDecoder;
use super::vision::{GitProjection, GitVision};

pub struct GitModel {
    vision: GitVision,
    projection: GitProjection,
    text: GitDecoder,
    output: Linear,
}

impl GitModel {
    pub fn new(config: &GitConfig, vars: VarBuilder) -> nim_error::Result<Self> {
        let git = vars.pp("git");

        Ok(Self {
            vision: GitVision::new(&config.vision_config, git.pp("image_encoder").pp("vision_model"))?,
            projection: GitProjection::new(
                config.hidden_size,
                config.vision_config.layer_norm_eps,
                git.pp("visual_projection").pp("visual_projection"),
            )?,
            text: GitDecoder::new(config, git)?,
            output: linear(config.hidden_size, config.vocab_size, vars.pp("output"))?,
        })
    }

    pub fn vision(&mut self, image: &Tensor) -> nim_error::Result<Tensor> {
        self.vision
            .forward(image)
            .map_err(Into::into)
            .and_then(|xs| self.projection.forward(&xs).map_err(Into::into))
    }

    pub fn logits(&mut self, visual: &Tensor, tokens: &[u32]) -> nim_error::Result<Tensor> {
        self.text
            .forward(visual, tokens)
            .and_then(|xs| xs.apply(&self.output).map_err(Into::into))
            .and_then(|xs| xs.i((.., xs.dim(1)? - 1, ..)).map_err(Into::into))
    }
}
