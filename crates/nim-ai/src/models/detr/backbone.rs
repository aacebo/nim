use candle_core::{Result, Tensor};
use candle_nn::{Conv2d, Conv2dConfig, Module, VarBuilder, conv2d_no_bias};

struct FrozenBatchNorm {
    scale: Tensor,
    bias: Tensor,
}

impl FrozenBatchNorm {
    fn new(channels: usize, vars: VarBuilder) -> Result<Self> {
        let weight = vars.get(channels, "weight")?;
        let bias = vars.get(channels, "bias")?;
        let mean = vars.get(channels, "running_mean")?;
        let variance = vars.get(channels, "running_var")?;
        let scale = weight.div(&(variance + 1e-5)?.sqrt()?)?;
        let bias = (bias - (&mean * &scale)?)?;

        Ok(Self {
            scale: scale.reshape((1, channels, 1, 1))?,
            bias: bias.reshape((1, channels, 1, 1))?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        input.broadcast_mul(&self.scale)?.broadcast_add(&self.bias)
    }
}

fn convolution(input: usize, output: usize, kernel: usize, stride: usize, vars: VarBuilder) -> Result<Conv2d> {
    conv2d_no_bias(
        input,
        output,
        kernel,
        Conv2dConfig {
            padding: kernel / 2,
            stride,
            ..Default::default()
        },
        vars,
    )
}

struct Bottleneck {
    conv1: Conv2d,
    conv2: Conv2d,
    conv3: Conv2d,
    bn1: FrozenBatchNorm,
    bn2: FrozenBatchNorm,
    bn3: FrozenBatchNorm,
    downsample: Option<(Conv2d, FrozenBatchNorm)>,
}

impl Bottleneck {
    fn new(input: usize, channels: usize, stride: usize, vars: VarBuilder) -> Result<Self> {
        let output = channels * 4;

        Ok(Self {
            conv1: convolution(input, channels, 1, 1, vars.pp("conv1"))?,
            conv2: convolution(channels, channels, 3, stride, vars.pp("conv2"))?,
            conv3: convolution(channels, output, 1, 1, vars.pp("conv3"))?,
            bn1: FrozenBatchNorm::new(channels, vars.pp("bn1"))?,
            bn2: FrozenBatchNorm::new(channels, vars.pp("bn2"))?,
            bn3: FrozenBatchNorm::new(output, vars.pp("bn3"))?,
            downsample: if stride != 1 || input != output {
                Some((
                    convolution(input, output, 1, stride, vars.pp("downsample.0"))?,
                    FrozenBatchNorm::new(output, vars.pp("downsample.1"))?,
                ))
            } else {
                None
            },
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let output = self.bn1.forward(&self.conv1.forward(input)?)?.relu()?;
        let output = self.bn2.forward(&self.conv2.forward(&output)?)?.relu()?;
        let output = self.bn3.forward(&self.conv3.forward(&output)?)?;
        let residual = match &self.downsample {
            Some((conv, norm)) => norm.forward(&conv.forward(input)?)?,
            None => input.clone(),
        };

        (output + residual)?.relu()
    }
}

pub struct Backbone {
    conv: Conv2d,
    norm: FrozenBatchNorm,
    blocks: Vec<Bottleneck>,
}

impl Backbone {
    pub fn new(vars: VarBuilder) -> Result<Self> {
        let mut blocks = Vec::new();
        let mut input = 64;

        for (stage, (channels, count)) in [(64, 3), (128, 4), (256, 6), (512, 3)].into_iter().enumerate() {
            for index in 0..count {
                let stride = if stage > 0 && index == 0 { 2 } else { 1 };

                blocks.push(Bottleneck::new(
                    input,
                    channels,
                    stride,
                    vars.pp(format!("layer{}.{}", stage + 1, index)),
                )?);

                input = channels * 4;
            }
        }

        Ok(Self {
            conv: convolution(3, 64, 7, 2, vars.pp("conv1"))?,
            norm: FrozenBatchNorm::new(64, vars.pp("bn1"))?,
            blocks,
        })
    }

    pub fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut output = self
            .norm
            .forward(&self.conv.forward(input)?)?
            .relu()?
            .pad_with_zeros(2, 1, 1)?
            .pad_with_zeros(3, 1, 1)?
            .max_pool2d_with_stride(3, 2)?;

        for block in &self.blocks {
            output = block.forward(&output)?;
        }

        Ok(output)
    }
}
