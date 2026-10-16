use candle_core::Tensor;
use candle_nn::{
    Conv2d, Conv2dConfig, Embedding, LayerNorm, Linear, Module, VarBuilder, conv2d_no_bias, embedding, layer_norm, linear,
};

use super::attention::Attention;
use super::config::VisionConfig;

pub struct Projection {
    linear: Linear,
    norm: LayerNorm,
}

impl Projection {
    pub fn new(hidden: usize, eps: f64, vars: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            linear: linear(hidden, hidden, vars.pp(0))?,
            norm: layer_norm(hidden, eps, vars.pp(1))?,
        })
    }
    pub fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        xs.apply(&self.linear)?.apply(&self.norm)
    }
}

pub struct Vision {
    embeddings: VisionEmbeddings,
    pre: LayerNorm,
    layers: Vec<VisionLayer>,
    post: LayerNorm,
}

impl Vision {
    pub fn new(c: &VisionConfig, vars: VarBuilder) -> candle_core::Result<Self> {
        let mut layers = Vec::with_capacity(c.num_hidden_layers);

        for i in 0..c.num_hidden_layers {
            layers.push(VisionLayer::new(c, vars.pp("encoder").pp("layers").pp(i))?);
        }

        Ok(Self {
            embeddings: VisionEmbeddings::new(c, vars.pp("embeddings"))?,
            pre: layer_norm(c.hidden_size, c.layer_norm_eps, vars.pp("pre_layrnorm"))?,
            layers,
            post: layer_norm(c.hidden_size, c.layer_norm_eps, vars.pp("post_layernorm"))?,
        })
    }

    pub fn forward(&mut self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let mut xs = self.embeddings.forward(xs)?.apply(&self.pre)?;

        for layer in &self.layers {
            xs = layer.forward(&xs)?;
        }

        xs.apply(&self.post)
    }
}

struct VisionEmbeddings {
    class: Tensor,
    patch: Conv2d,
    position: Embedding,
    hidden: usize,
}

impl VisionEmbeddings {
    fn new(c: &VisionConfig, vars: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            class: vars.get(c.hidden_size, "class_embedding")?,
            patch: conv2d_no_bias(
                c.num_channels,
                c.hidden_size,
                c.patch_size,
                Conv2dConfig {
                    stride: c.patch_size,
                    ..Default::default()
                },
                vars.pp("patch_embedding"),
            )?,
            position: embedding(
                (c.image_size / c.patch_size).pow(2) + 1,
                c.hidden_size,
                vars.pp("position_embedding"),
            )?,
            hidden: c.hidden_size,
        })
    }

    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let batch = xs.dim(0)?;
        let patches = self.patch.forward(xs)?.flatten_from(2)?.transpose(1, 2)?;
        let class = self
            .class
            .reshape((1, 1, self.hidden))?
            .broadcast_as((batch, 1, self.hidden))?;
        let xs = Tensor::cat(&[&class, &patches], 1)?;
        let ids = Tensor::arange(0u32, xs.dim(1)? as u32, xs.device())?.unsqueeze(0)?;
        xs.broadcast_add(&self.position.forward(&ids)?)
    }
}

struct VisionLayer {
    attention: Attention,
    norm1: LayerNorm,
    fc1: Linear,
    fc2: Linear,
    norm2: LayerNorm,
}

impl VisionLayer {
    fn new(c: &VisionConfig, vars: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            attention: Attention::new(
                c.hidden_size,
                c.num_attention_heads,
                vars.pp("self_attn"),
                "q_proj",
                "k_proj",
                "v_proj",
                "out_proj",
            )?,
            norm1: layer_norm(c.hidden_size, c.layer_norm_eps, vars.pp("layer_norm1"))?,
            fc1: linear(c.hidden_size, c.intermediate_size, vars.pp("mlp").pp("fc1"))?,
            fc2: linear(c.intermediate_size, c.hidden_size, vars.pp("mlp").pp("fc2"))?,
            norm2: layer_norm(c.hidden_size, c.layer_norm_eps, vars.pp("layer_norm2"))?,
        })
    }

    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let xs = (xs + self.attention.forward(&xs.apply(&self.norm1)?, None)?)?;
        let residual = xs.clone();

        xs.apply(&self.norm2)?
            .apply(&self.fc1)?
            .gelu()?
            .apply(&self.fc2)?
            .broadcast_add(&residual)
    }
}
