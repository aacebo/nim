use candle_core::{DType, Device, Tensor};
use candle_nn::{Conv2d, LayerNorm, Linear, Module, VarBuilder, conv2d, layer_norm, linear, ops};

use super::Config;
use super::backbone::Backbone;
use super::transformer::{DecoderLayer, EncoderLayer};
use crate::Error;

pub struct Model {
    backbone: Backbone,
    projection: Conv2d,
    queries: Tensor,
    encoder: Vec<EncoderLayer>,
    decoder: Vec<DecoderLayer>,
    norm: LayerNorm,
    classifier: Linear,
    boxes: [Linear; 3],
    dim: usize,
}

impl Model {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        config.validate()?;
        let model = vars.pp("model");

        Ok(Self {
            backbone: Backbone::new(model.pp("backbone.conv_encoder.model"))?,
            projection: conv2d(2048, config.d_model, 1, Default::default(), model.pp("input_projection"))?,
            queries: model
                .pp("query_position_embeddings")
                .get((config.num_queries, config.d_model), "weight")?
                .unsqueeze(0)?,
            encoder: (0..config.encoder_layers)
                .map(|i| EncoderLayer::new(config, model.pp(format!("encoder.layers.{i}"))))
                .collect::<candle_core::Result<_>>()?,
            decoder: (0..config.decoder_layers)
                .map(|i| DecoderLayer::new(config, model.pp(format!("decoder.layers.{i}"))))
                .collect::<candle_core::Result<_>>()?,
            norm: layer_norm(config.d_model, 1e-5, model.pp("decoder.layernorm"))?,
            classifier: linear(config.d_model, config.num_classes() + 1, vars.pp("class_labels_classifier"))?,
            boxes: [
                linear(config.d_model, config.d_model, vars.pp("bbox_predictor.layers.0"))?,
                linear(config.d_model, config.d_model, vars.pp("bbox_predictor.layers.1"))?,
                linear(config.d_model, 4, vars.pp("bbox_predictor.layers.2"))?,
            ],
            dim: config.d_model,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<(Tensor, Tensor), Error> {
        let (batch, channels, height, width) = input.dims4()?;

        if batch != 1 || channels != 3 || height == 0 || width == 0 {
            return Err(Error::custom("ai::image", "DETR expects one nonempty RGB image"));
        }

        let features = self.backbone.forward(input)?;
        let (_, _, height, width) = features.dims4()?;
        let positions = positions(height, width, self.dim, input.device())?.to_dtype(input.dtype())?;
        let mut memory = self
            .projection
            .forward(&features)?
            .flatten_from(2)?
            .transpose(1, 2)?
            .contiguous()?;

        for layer in &self.encoder {
            memory = layer.forward(&memory, &positions)?;
        }

        let mut output = self.queries.zeros_like()?;

        for layer in &self.decoder {
            output = layer.forward(&output, &self.queries, &memory, &positions)?;
        }

        let output = self.norm.forward(&output)?;
        let logits = self.classifier.forward(&output)?;
        let boxes = self.boxes[0].forward(&output)?.relu()?;
        let boxes = self.boxes[1].forward(&boxes)?.relu()?;
        let boxes = ops::sigmoid(&self.boxes[2].forward(&boxes)?)?;
        Ok((logits, boxes))
    }
}

fn positions(height: usize, width: usize, dim: usize, device: &Device) -> candle_core::Result<Tensor> {
    let half = dim / 2;
    let mut values = Vec::with_capacity(height * width * dim);

    for y in 0..height {
        for x in 0..width {
            for coordinate in [
                (y + 1) as f32 / (height as f32 + 1e-6),
                (x + 1) as f32 / (width as f32 + 1e-6),
            ] {
                for channel in 0..half {
                    let divisor = 10000f32.powf((2 * (channel / 2)) as f32 / half as f32);
                    let angle = (coordinate * std::f32::consts::TAU) / divisor;
                    values.push(if channel % 2 == 0 { angle.sin() } else { angle.cos() });
                }
            }
        }
    }

    Tensor::from_vec(values, (1, height * width, dim), device)?.to_dtype(DType::F32)
}
