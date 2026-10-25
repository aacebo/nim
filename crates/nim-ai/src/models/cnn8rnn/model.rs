use candle_core::{DType, IndexOp, Tensor};
use candle_nn::{BatchNorm, Conv2d, Conv2dConfig, Linear, VarBuilder, batch_norm, conv2d_no_bias, linear};

use crate::Error;

use super::config::Config;
use super::processor::Processor;

pub struct Model {
    processor: Processor,
    bn: BatchNorm,
    blocks: Vec<ConvBlock>,
    fc: Linear,
    forward: Gru,
    backward: Gru,
    classifier: Linear,
}

struct ConvBlock {
    conv1: Conv2d,
    conv2: Conv2d,
    bn1: BatchNorm,
    bn2: BatchNorm,
}

struct Gru {
    input: Tensor,
    hidden: Tensor,
    input_bias: Tensor,
    hidden_bias: Tensor,
}

impl Model {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        let filter = vars.get((513, 64), "melspec_extractor.mel_scale.fb")?.to_vec2::<f32>()?;
        let window = vars.get(1024, "melspec_extractor.spectrogram.window")?.to_vec1::<f32>()?;

        Ok(Self {
            processor: Processor::new(filter, window),
            bn: batch_norm(64, 1e-5, vars.pp("bn0"))?,
            blocks: [(1, 64), (64, 128), (128, 256), (256, 512)]
                .into_iter()
                .enumerate()
                .map(|(index, (input, output))| ConvBlock::new(input, output, vars.pp(format!("conv_block{}", index + 1))))
                .collect::<Result<_, _>>()?,
            fc: linear(512, 512, vars.pp("fc1"))?,
            forward: Gru::new(false, vars.pp("rnn"))?,
            backward: Gru::new(true, vars.pp("rnn"))?,
            classifier: linear(512, config.classes_num, vars.pp("fc_audioset"))?,
        })
    }

    pub fn forward(&self, bytes: &[u8], config: &Config, device: &candle_core::Device) -> Result<(Tensor, f64), Error> {
        let (features, duration) = self.processor.features(bytes, config)?;
        let frames = features.len();
        let values = features.into_iter().flatten().collect::<Vec<_>>();
        let mut output = Tensor::from_vec(values, (1, frames, 64), device)?
            .unsqueeze(1)?
            .transpose(1, 3)?
            .apply_t(&self.bn, false)?
            .transpose(1, 3)?;

        for (index, block) in self.blocks.iter().enumerate() {
            output = block.forward(&output)?;
            let pool = if index < 2 { (2, 2) } else { (1, 2) };
            output = (&output.avg_pool2d(pool)? + &output.max_pool2d(pool)?)?;
        }

        let output = output.mean(3)?.transpose(1, 2)?.apply(&self.fc)?.relu()?;
        let left = self.forward.forward(&output)?;
        let right = self.backward.forward(&output.contiguous()?.flip(&[1])?)?.flip(&[1])?;
        let output = candle_nn::ops::sigmoid(&Tensor::cat(&[left, right], 2)?.apply(&self.classifier)?)?;

        Ok((output.sqr()?.sum(1)?.broadcast_div(&output.sum(1)?)?, duration))
    }
}

impl ConvBlock {
    fn new(input: usize, output: usize, vars: VarBuilder) -> candle_core::Result<Self> {
        let config = Conv2dConfig {
            padding: 1,
            ..Default::default()
        };

        Ok(Self {
            conv1: conv2d_no_bias(input, output, 3, config, vars.pp("conv1"))?,
            conv2: conv2d_no_bias(output, output, 3, config, vars.pp("conv2"))?,
            bn1: batch_norm(output, 1e-5, vars.pp("bn1"))?,
            bn2: batch_norm(output, 1e-5, vars.pp("bn2"))?,
        })
    }

    fn forward(&self, input: &Tensor) -> candle_core::Result<Tensor> {
        input
            .apply(&self.conv1)?
            .apply_t(&self.bn1, false)?
            .relu()?
            .apply(&self.conv2)?
            .apply_t(&self.bn2, false)?
            .relu()
    }
}

impl Gru {
    fn new(reverse: bool, vars: VarBuilder) -> candle_core::Result<Self> {
        let suffix = if reverse { "_l0_reverse" } else { "_l0" };

        Ok(Self {
            input: vars.get((768, 512), &format!("weight_ih{suffix}"))?,
            hidden: vars.get((768, 256), &format!("weight_hh{suffix}"))?,
            input_bias: vars.get(768, &format!("bias_ih{suffix}"))?,
            hidden_bias: vars.get(768, &format!("bias_hh{suffix}"))?,
        })
    }

    fn forward(&self, input: &Tensor) -> candle_core::Result<Tensor> {
        let (batch, frames, _) = input.dims3()?;
        let mut state = Tensor::zeros((batch, 256), DType::F32, input.device())?;
        let mut output = Vec::with_capacity(frames);

        for frame in 0..frames {
            let input = input.i((.., frame, ..))?.contiguous()?;
            let input = input.matmul(&self.input.t()?)?.broadcast_add(&self.input_bias)?;
            let hidden = state.matmul(&self.hidden.t()?)?.broadcast_add(&self.hidden_bias)?;
            let input = input.chunk(3, 1)?;
            let hidden = hidden.chunk(3, 1)?;
            let reset = candle_nn::ops::sigmoid(&(&input[0] + &hidden[0])?)?;
            let update = candle_nn::ops::sigmoid(&(&input[1] + &hidden[1])?)?;
            let next = (&input[2] + (reset * &hidden[2])?)?.tanh()?;
            state = (&update * &state - ((&update - 1.0)? * next)?)?;
            output.push(state.clone());
        }

        Tensor::stack(&output, 1)
    }
}
