use candle_core::{Result, Tensor};
use candle_nn::{BatchNorm, Conv2d, Conv2dConfig, Linear, Module, VarBuilder, batch_norm, conv2d_no_bias, linear};

use super::Config;

pub struct MobileNetV2 {
    stem: Stem,
    layers: Vec<Block>,
    head: Layer,
    classifier: Linear,
}

impl MobileNetV2 {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self> {
        let channels =
            [16, 24, 24, 32, 32, 32, 64, 64, 64, 64, 96, 96, 96, 160, 160, 160, 320].map(|channels| config.channels(channels));
        let strides = [2, 1, 2, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1];
        let stem = Stem::new(
            vars.pp("mobilenet_v2.conv_stem"),
            config,
            config.num_channels,
            config.channels(32),
            channels[0],
        )?;

        let mut current_stride = 2;
        let mut layers = Vec::with_capacity(strides.len());

        for (index, stride) in strides.into_iter().enumerate() {
            let stride = if current_stride == config.output_stride { 1 } else { stride };

            current_stride *= stride;

            layers.push(Block::new(
                vars.pp(format!("mobilenet_v2.layer.{index}")),
                config,
                channels[index],
                channels[index + 1],
                stride,
            )?);
        }

        let outputs = config.output_channels();

        Ok(Self {
            stem,
            layers,
            head: Layer::new(vars.pp("mobilenet_v2.conv_1x1"), config, channels[16], outputs, 1, 1, 1, true)?,
            classifier: linear(outputs, config.id2label.len(), vars.pp("classifier"))?,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut output = self.stem.forward(input)?;
        for layer in &self.layers {
            output = layer.forward(&output)?;
        }

        self.classifier.forward(&self.head.forward(&output)?.mean((2, 3))?)
    }
}

struct Stem {
    first: Layer,
    expand: Option<Layer>,
    depthwise: Layer,
    reduce: Layer,
}

impl Stem {
    fn new(vars: VarBuilder, c: &Config, input: usize, expanded: usize, output: usize) -> Result<Self> {
        Ok(Self {
            first: Layer::new(vars.pp("first_conv"), c, input, expanded, 3, 2, 1, true)?,
            expand: if c.first_layer_is_expansion {
                None
            } else {
                Some(Layer::new(vars.pp("expand_1x1"), c, expanded, expanded, 1, 1, 1, true)?)
            },
            depthwise: Layer::new(vars.pp("conv_3x3"), c, expanded, expanded, 3, 1, expanded, true)?,
            reduce: Layer::new(vars.pp("reduce_1x1"), c, expanded, output, 1, 1, 1, false)?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut output = self.first.forward(input)?;

        if let Some(expand) = &self.expand {
            output = expand.forward(&output)?;
        }

        self.reduce.forward(&self.depthwise.forward(&output)?)
    }
}

struct Block {
    expand: Layer,
    depthwise: Layer,
    reduce: Layer,
    residual: bool,
}

impl Block {
    fn new(vars: VarBuilder, c: &Config, input: usize, output: usize, stride: usize) -> Result<Self> {
        let expanded = c.channels((input as f64 * c.expand_ratio).round() as usize);

        Ok(Self {
            expand: Layer::new(vars.pp("expand_1x1"), c, input, expanded, 1, 1, 1, true)?,
            depthwise: Layer::new(vars.pp("conv_3x3"), c, expanded, expanded, 3, stride, expanded, true)?,
            reduce: Layer::new(vars.pp("reduce_1x1"), c, expanded, output, 1, 1, 1, false)?,
            residual: stride == 1 && input == output,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let output = self.reduce.forward(&self.depthwise.forward(&self.expand.forward(input)?)?)?;

        if self.residual {
            return input + output;
        }

        Ok(output)
    }
}

struct Layer {
    convolution: Conv2d,
    normalization: BatchNorm,
    activation: bool,
    kernel: usize,
    stride: usize,
    tf_padding: bool,
}

impl Layer {
    fn new(
        vars: VarBuilder,
        c: &Config,
        input: usize,
        output: usize,
        kernel: usize,
        stride: usize,
        groups: usize,
        activation: bool,
    ) -> Result<Self> {
        Ok(Self {
            convolution: conv2d_no_bias(
                input,
                output,
                kernel,
                Conv2dConfig {
                    stride,
                    groups,
                    ..Default::default()
                },
                vars.pp("convolution"),
            )?,
            normalization: batch_norm(output, c.layer_norm_eps, vars.pp("normalization"))?,
            activation,
            kernel,
            stride,
            tf_padding: c.tf_padding,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let input = if self.tf_padding {
            pad_same(input, self.kernel, self.stride)?
        } else {
            input.clone()
        };

        let output = input.apply(&self.convolution)?.apply_t(&self.normalization, false)?;

        if self.activation {
            output.clamp(0f32, 6f32)
        } else {
            Ok(output)
        }
    }
}

fn pad_same(input: &Tensor, kernel: usize, stride: usize) -> Result<Tensor> {
    let (_, _, height, width) = input.dims4()?;
    let padding = |length: usize| {
        if length % stride == 0 {
            kernel.saturating_sub(stride)
        } else {
            kernel.saturating_sub(length % stride)
        }
    };
    let vertical = padding(height);
    let horizontal = padding(width);

    input
        .pad_with_zeros(2, vertical / 2, vertical - vertical / 2)?
        .pad_with_zeros(3, horizontal / 2, horizontal - horizontal / 2)
}
