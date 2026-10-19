use candle_core::{Result, Tensor};
use candle_nn::{LayerNorm, Linear, Module, VarBuilder, layer_norm, linear, ops};

use super::Config;

struct Attention {
    query: Linear,
    key: Linear,
    value: Linear,
    output: Linear,
    heads: usize,
    head_dim: usize,
}

impl Attention {
    fn new(dim: usize, heads: usize, vars: VarBuilder) -> Result<Self> {
        Ok(Self {
            query: linear(dim, dim, vars.pp("q_proj"))?,
            key: linear(dim, dim, vars.pp("k_proj"))?,
            value: linear(dim, dim, vars.pp("v_proj"))?,
            output: linear(dim, dim, vars.pp("out_proj"))?,
            heads,
            head_dim: dim / heads,
        })
    }

    fn split(&self, input: &Tensor) -> Result<Tensor> {
        let (batch, seq, _) = input.dims3()?;
        input
            .reshape((batch, seq, self.heads, self.head_dim))?
            .transpose(1, 2)?
            .contiguous()
    }

    fn forward(&self, queries: &Tensor, keys: &Tensor, values: &Tensor) -> Result<Tensor> {
        let query = self.split(&self.query.forward(queries)?)?;
        let query = (query * (self.head_dim as f64).powf(-0.5))?;
        let key = self.split(&self.key.forward(keys)?)?;
        let value = self.split(&self.value.forward(values)?)?;
        let probabilities = ops::softmax_last_dim(&query.matmul(&key.t()?)?)?;
        let (batch, _, seq, _) = query.dims4()?;
        let output =
            probabilities
                .matmul(&value)?
                .transpose(1, 2)?
                .contiguous()?
                .reshape((batch, seq, self.heads * self.head_dim))?;
        self.output.forward(&output)
    }
}

pub struct EncoderLayer {
    attention: Attention,
    attention_norm: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    final_norm: LayerNorm,
}

impl EncoderLayer {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self> {
        Ok(Self {
            attention: Attention::new(config.d_model, config.encoder_attention_heads, vars.pp("self_attn"))?,
            attention_norm: layer_norm(config.d_model, 1e-5, vars.pp("self_attn_layer_norm"))?,
            fc1: linear(config.d_model, config.encoder_ffn_dim, vars.pp("fc1"))?,
            fc2: linear(config.encoder_ffn_dim, config.d_model, vars.pp("fc2"))?,
            final_norm: layer_norm(config.d_model, 1e-5, vars.pp("final_layer_norm"))?,
        })
    }

    pub fn forward(&self, input: &Tensor, positions: &Tensor) -> Result<Tensor> {
        let query = (input + positions)?;
        let attended = self.attention.forward(&query, &query, input)?;
        let output = self.attention_norm.forward(&(input + attended)?)?;
        let feedforward = self.fc2.forward(&self.fc1.forward(&output)?.relu()?)?;
        self.final_norm.forward(&(output + feedforward)?)
    }
}

pub struct DecoderLayer {
    attention: Attention,
    attention_norm: LayerNorm,
    cross: Attention,
    cross_norm: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    final_norm: LayerNorm,
}

impl DecoderLayer {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self> {
        Ok(Self {
            attention: Attention::new(config.d_model, config.decoder_attention_heads, vars.pp("self_attn"))?,
            attention_norm: layer_norm(config.d_model, 1e-5, vars.pp("self_attn_layer_norm"))?,
            cross: Attention::new(config.d_model, config.decoder_attention_heads, vars.pp("encoder_attn"))?,
            cross_norm: layer_norm(config.d_model, 1e-5, vars.pp("encoder_attn_layer_norm"))?,
            fc1: linear(config.d_model, config.decoder_ffn_dim, vars.pp("fc1"))?,
            fc2: linear(config.decoder_ffn_dim, config.d_model, vars.pp("fc2"))?,
            final_norm: layer_norm(config.d_model, 1e-5, vars.pp("final_layer_norm"))?,
        })
    }

    pub fn forward(&self, input: &Tensor, queries: &Tensor, memory: &Tensor, positions: &Tensor) -> Result<Tensor> {
        let query = (input + queries)?;
        let attended = self.attention.forward(&query, &query, input)?;
        let output = self.attention_norm.forward(&(input + attended)?)?;
        let attended = self.cross.forward(&(&output + queries)?, &(memory + positions)?, memory)?;
        let output = self.cross_norm.forward(&(output + attended)?)?;
        let feedforward = self.fc2.forward(&self.fc1.forward(&output)?.relu()?)?;
        self.final_norm.forward(&(output + feedforward)?)
    }
}
