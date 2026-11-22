use candle_core::{Result, Tensor};
use candle_nn::{Embedding, LayerNorm, Linear, VarBuilder, layer_norm, linear};

use super::attention::BartAttention;
use super::config::BartConfig;
use super::embeddings::BartLearnedPositionalEmbedding;

#[derive(Debug, Clone)]
pub struct BartEncoder {
    embed_tokens: Embedding,
    embed_positions: BartLearnedPositionalEmbedding,
    layernorm_embedding: LayerNorm,
    layers: Vec<BartEncoderLayer>,
    embed_scale: Option<f64>,
}

impl BartEncoder {
    pub fn new(cfg: &BartConfig, embed_tokens: &Embedding, vb: VarBuilder) -> Result<Self> {
        let mut layers = Vec::with_capacity(cfg.encoder_layers);
        let vb_l = vb.pp("layers");

        for index in 0..cfg.encoder_layers {
            layers.push(BartEncoderLayer::new(cfg, vb_l.pp(index))?);
        }

        Ok(Self {
            embed_tokens: embed_tokens.clone(),
            embed_positions: BartLearnedPositionalEmbedding::new(cfg, vb.pp("embed_positions"))?,
            layernorm_embedding: layer_norm(cfg.d_model, 1e-5, vb.pp("layernorm_embedding"))?,
            layers,
            embed_scale: cfg.scale_embedding.then(|| (cfg.d_model as f64).sqrt()),
        })
    }

    pub fn forward(&mut self, xs: &Tensor) -> Result<Tensor> {
        let seq_len = xs.dim(1)?;
        let xs = xs.apply(&self.embed_tokens)?;
        let xs = match self.embed_scale {
            None => xs,
            Some(scale) => (xs * scale)?,
        };

        let positions = self.embed_positions.forward(seq_len, 0, xs.device())?.unsqueeze(0)?;
        let mut xs = xs.broadcast_add(&positions)?.apply(&self.layernorm_embedding)?;

        for layer in self.layers.iter_mut() {
            xs = layer.forward(&xs)?;
        }

        Ok(xs)
    }
}

#[derive(Debug, Clone)]
pub struct BartEncoderLayer {
    self_attn: BartAttention,
    self_attn_layer_norm: LayerNorm,
    activation_fn: candle_nn::Activation,
    fc1: Linear,
    fc2: Linear,
    final_layer_norm: LayerNorm,
}

impl BartEncoderLayer {
    pub fn new(cfg: &BartConfig, vb: VarBuilder) -> Result<Self> {
        Ok(Self {
            self_attn: BartAttention::new(cfg, false, vb.pp("self_attn"))?,
            self_attn_layer_norm: layer_norm(cfg.d_model, 1e-5, vb.pp("self_attn_layer_norm"))?,
            activation_fn: cfg.activation_function,
            fc1: linear(cfg.d_model, cfg.encoder_ffn_dim, vb.pp("fc1"))?,
            fc2: linear(cfg.encoder_ffn_dim, cfg.d_model, vb.pp("fc2"))?,
            final_layer_norm: layer_norm(cfg.d_model, 1e-5, vb.pp("final_layer_norm"))?,
        })
    }

    pub fn forward(&mut self, xs: &Tensor) -> Result<Tensor> {
        let residual = xs;
        let xs = (self.self_attn.forward(xs, None, None)? + residual)?.apply(&self.self_attn_layer_norm)?;
        let residual = &xs;
        let xs = xs.apply(&self.fc1)?.apply(&self.activation_fn)?.apply(&self.fc2)?;

        (xs + residual)?.apply(&self.final_layer_norm)
    }
}
