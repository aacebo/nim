use candle_core::{DType, IndexOp, Tensor};
use candle_nn::{
    BatchNorm, Conv2d, Conv2dConfig, GRU, GRUConfig, Linear, Module, RNN, VarBuilder, batch_norm, conv2d_no_bias, embedding, gru,
    linear,
};

use crate::Error;

use super::config::Config;
use super::processor::Processor;

pub struct Model {
    processor: Processor,
    cnn: Cnn,
    encoder: Vec<(Gru, Gru)>,
    decoder: Decoder,
}

struct ConvBlock {
    conv1: Conv2d,
    conv2: Conv2d,
    bn1: BatchNorm,
    bn2: BatchNorm,
}

struct Cnn {
    bn: BatchNorm,
    blocks: Vec<ConvBlock>,
}

struct Decoder {
    words: candle_nn::Embedding,
    temporal: candle_nn::Embedding,
    attention: Attention,
    context: Linear,
    features: Linear,
    gru: GRU,
    classifier: Linear,
}

struct Attention {
    projection: Linear,
    vector: Tensor,
}

struct Gru {
    input: Tensor,
    hidden: Tensor,
    input_bias: Tensor,
    hidden_bias: Tensor,
    size: usize,
}

impl Model {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        let filter = vars.get((513, 64), "melspec_extractor.mel_scale.fb")?.to_vec2::<f32>()?;
        let window = vars.get(1024, "melspec_extractor.spectrogram.window")?.to_vec1::<f32>()?;
        let cnn = Cnn::new(vars.pp("cap_model.encoder.cnn"))?;
        let encoder_vars = vars.pp("cap_model.encoder.rnn.network");
        let encoder = (0..config.encoder_rnn_num_layers)
            .map(|layer| {
                Ok((
                    Gru::new(config.encoder_rnn_hidden_size, layer, false, encoder_vars.clone())?,
                    Gru::new(config.encoder_rnn_hidden_size, layer, true, encoder_vars.clone())?,
                ))
            })
            .collect::<Result<Vec<_>, candle_core::Error>>()?;

        Ok(Self {
            processor: Processor::new(filter, window),
            cnn,
            encoder,
            decoder: Decoder::new(config, vars.pp("cap_model.decoder"))?,
        })
    }

    pub fn chunks(&self, bytes: &[u8], config: &Config) -> Result<Vec<Vec<Vec<f32>>>, Error> {
        self.processor.chunks(bytes, config)
    }

    pub fn encode(&self, features: Vec<Vec<f32>>, device: &candle_core::Device) -> Result<Tensor, Error> {
        let frames = features.len();
        let values = features.into_iter().flatten().collect::<Vec<_>>();
        let input = Tensor::from_vec(values, (1, frames, 64), device)?.transpose(1, 2)?;
        let mut output = self.cnn.forward(&input)?;

        for (forward, backward) in &self.encoder {
            let left = forward.forward(&output)?;
            let right = backward.forward(&output.contiguous()?.flip(&[1])?)?.flip(&[1])?;
            output = Tensor::cat(&[left, right], 2)?;
        }

        Ok(output)
    }

    pub fn logits(&self, audio: &Tensor, tokens: &[u32]) -> Result<Tensor, Error> {
        let mut state = None;
        let mut output = None;

        for (index, token) in tokens.iter().enumerate() {
            let input = if index == 0 {
                self.decoder.temporal.forward(&Tensor::new(&[3u32], audio.device())?)?
            } else {
                self.decoder.words.forward(&Tensor::new(&[*token], audio.device())?)?
            };
            let (next, logits) = self.decoder.step(audio, &input, state.as_ref())?;
            state = Some(next);
            output = Some(logits);
        }

        output.ok_or_else(|| Error::custom("ai::audio", "caption decoder received no tokens"))
    }
}

impl Gru {
    fn new(size: usize, layer: usize, reverse: bool, vars: VarBuilder) -> candle_core::Result<Self> {
        let suffix = if reverse {
            format!("_l{layer}_reverse")
        } else {
            format!("_l{layer}")
        };

        Ok(Self {
            input: vars.get(
                (size * 3, if layer == 0 { 2048 } else { size * 2 }),
                &format!("weight_ih{suffix}"),
            )?,
            hidden: vars.get((size * 3, size), &format!("weight_hh{suffix}"))?,
            input_bias: vars.get(size * 3, &format!("bias_ih{suffix}"))?,
            hidden_bias: vars.get(size * 3, &format!("bias_hh{suffix}"))?,
            size,
        })
    }

    fn forward(&self, input: &Tensor) -> candle_core::Result<Tensor> {
        let (batch, frames, _) = input.dims3()?;
        let mut state = Tensor::zeros((batch, self.size), DType::F32, input.device())?;
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

impl Cnn {
    fn new(vars: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            bn: batch_norm(64, 1e-5, vars.pp("bn0"))?,
            blocks: [(1, 64), (64, 128), (128, 256), (256, 512), (512, 1024), (1024, 2048)]
                .into_iter()
                .enumerate()
                .map(|(index, (input, output))| ConvBlock::new(input, output, vars.pp(format!("conv_block{}", index + 1))))
                .collect::<Result<_, _>>()?,
        })
    }

    fn forward(&self, input: &Tensor) -> candle_core::Result<Tensor> {
        let mut output = input
            .transpose(1, 2)?
            .unsqueeze(1)?
            .transpose(1, 3)?
            .apply_t(&self.bn, false)?
            .transpose(1, 3)?;

        for (index, block) in self.blocks.iter().enumerate() {
            output = block.forward(&output)?;
            if index < 5 {
                output = output.avg_pool2d((2, 2))?;
            }
        }

        output.mean(3)?.transpose(1, 2)
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

impl Decoder {
    fn new(config: &Config, vars: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            words: embedding(config.vocab_size, config.decoder_emb_dim, vars.pp("word_embedding"))?,
            temporal: embedding(4, config.decoder_emb_dim, vars.pp("temporal_embedding"))?,
            attention: Attention::new(config.attn_emb_dim, config.decoder_d_model, vars.pp("attn"))?,
            context: linear(config.attn_emb_dim, config.decoder_emb_dim, vars.pp("ctx_proj"))?,
            features: linear(config.fc_emb_dim, config.decoder_emb_dim, vars.pp("fc_proj"))?,
            gru: gru(
                config.decoder_emb_dim * 3,
                config.decoder_d_model,
                GRUConfig::default(),
                vars.pp("model"),
            )?,
            classifier: linear(config.decoder_d_model, config.vocab_size, vars.pp("classifier"))?,
        })
    }

    fn step(
        &self,
        audio: &Tensor,
        token: &Tensor,
        state: Option<&candle_nn::rnn::GRUState>,
    ) -> candle_core::Result<(candle_nn::rnn::GRUState, Tensor)> {
        let state = match state {
            Some(state) => state.clone(),
            None => self.gru.zero_state(1)?,
        };
        let context = self.attention.forward(audio, state.h())?;
        let features = audio.mean(1)?.apply(&self.features)?;
        let context = context.apply(&self.context)?;
        let input = Tensor::cat(&[token, &context, &features], 1)?;
        let state = self.gru.step(&input, &state)?;
        let logits = state.h().apply(&self.classifier)?;

        Ok((state, logits.squeeze(0)?))
    }
}

impl Attention {
    fn new(encoder: usize, decoder: usize, vars: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            projection: linear(encoder + decoder, 512, vars.pp("h2attn"))?,
            vector: vars.get(512, "v")?,
        })
    }

    fn forward(&self, audio: &Tensor, state: &Tensor) -> candle_core::Result<Tensor> {
        let (_, frames, _) = audio.dims3()?;
        let state = state.unsqueeze(1)?.broadcast_as((1, frames, state.dim(1)?))?;
        let input = Tensor::cat(&[audio, &state], 2)?;
        let scores = input
            .apply(&self.projection)?
            .tanh()?
            .matmul(&self.vector.reshape((1, 512, 1))?)?
            .squeeze(2)?;
        let weights = candle_nn::ops::softmax_last_dim(&scores)?;

        weights.unsqueeze(1)?.matmul(audio)?.squeeze(1)
    }
}
