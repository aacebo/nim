use candle_core::{DType, Tensor};
use candle_nn::{Embedding, LayerNorm, Linear, Module, VarBuilder, embedding, layer_norm, linear};

use crate::Error;

use super::attention::Attention;
use super::config::Config;

pub struct Decoder {
    words: Embedding,
    positions: Embedding,
    norm: LayerNorm,
    layers: Vec<TextLayer>,
    image_tokens: usize,
}

impl Decoder {
    pub fn new(c: &Config, vars: VarBuilder) -> candle_core::Result<Self> {
        let mut layers = Vec::with_capacity(c.num_hidden_layers);

        for i in 0..c.num_hidden_layers {
            layers.push(TextLayer::new(c, vars.pp("encoder").pp("layer").pp(i))?);
        }

        Ok(Self {
            words: embedding(c.vocab_size, c.hidden_size, vars.pp("embeddings").pp("word_embeddings"))?,
            positions: embedding(
                c.max_position_embeddings,
                c.hidden_size,
                vars.pp("embeddings").pp("position_embeddings"),
            )?,
            norm: layer_norm(c.hidden_size, c.layer_norm_eps, vars.pp("embeddings").pp("LayerNorm"))?,
            layers,
            image_tokens: (c.vision_config.image_size / c.vision_config.patch_size).pow(2) + 1,
        })
    }

    pub fn forward(&mut self, visual: &Tensor, tokens: &[u32]) -> Result<Tensor, Error> {
        let ids = Tensor::from_slice(tokens, (1, tokens.len()), visual.device())?;
        let positions = Tensor::arange(0u32, tokens.len() as u32, visual.device())?.unsqueeze(0)?;
        let text = (self.words.forward(&ids)? + self.positions.forward(&positions)?)?.apply(&self.norm)?;
        let mut xs = Tensor::cat(&[visual, &text], 1)?;
        let len = xs.dim(1)?;
        let text_start = self.image_tokens;
        let values: Vec<f32> = (0..len)
            .flat_map(|row| {
                (0..len).map(move |col| {
                    if row < text_start || col <= row {
                        0.0
                    } else {
                        f32::NEG_INFINITY
                    }
                })
            })
            .collect();

        let mask = Tensor::from_vec(values, (len, len), visual.device())?.to_dtype(DType::F32)?;

        for layer in &self.layers {
            xs = layer.forward(&xs, &mask)?;
        }

        Ok(xs)
    }
}

struct TextLayer {
    attention: Attention,
    attention_norm: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    output_norm: LayerNorm,
}

impl TextLayer {
    fn new(c: &Config, vars: VarBuilder) -> candle_core::Result<Self> {
        let self_vars = vars.pp("attention").pp("self");

        Ok(Self {
            attention: Attention::from_parts(
                c.hidden_size,
                c.num_attention_heads,
                self_vars,
                vars.pp("attention").pp("output").pp("dense"),
            )?,
            attention_norm: layer_norm(
                c.hidden_size,
                c.layer_norm_eps,
                vars.pp("attention").pp("output").pp("LayerNorm"),
            )?,
            fc1: linear(c.hidden_size, c.intermediate_size, vars.pp("intermediate").pp("dense"))?,
            fc2: linear(c.intermediate_size, c.hidden_size, vars.pp("output").pp("dense"))?,
            output_norm: layer_norm(c.hidden_size, c.layer_norm_eps, vars.pp("output").pp("LayerNorm"))?,
        })
    }

    fn forward(&self, xs: &Tensor, mask: &Tensor) -> candle_core::Result<Tensor> {
        let xs = (xs + self.attention.forward(xs, Some(mask))?)?.apply(&self.attention_norm)?;
        let residual = xs.clone();

        xs.apply(&self.fc1)?
            .gelu_erf()?
            .apply(&self.fc2)?
            .broadcast_add(&residual)?
            .apply(&self.output_norm)
    }
}
