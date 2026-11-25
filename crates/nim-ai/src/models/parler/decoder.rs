use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{Embedding, Linear};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::parler_tts::{Decoder, DecoderConfig, Model};
use candle_transformers::models::t5::T5EncoderModel;

use super::config::ParlerConfig;

pub struct ParlerDecoder {
    decoder: Decoder,
    text_encoder: T5EncoderModel,
    embed_prompts: Embedding,
    enc_to_dec_proj: Option<Linear>,
    decoder_start_token_id: u32,
    pad_token_id: u32,
    config: DecoderConfig,
}

impl ParlerDecoder {
    pub fn new(config: &ParlerConfig, model: Model) -> Self {
        Self {
            decoder: model.decoder,
            text_encoder: model.text_encoder,
            embed_prompts: model.embed_prompts,
            enc_to_dec_proj: model.enc_to_dec_proj,
            decoder_start_token_id: model.decoder_start_token_id,
            pad_token_id: model.pad_token_id,
            config: config.model.decoder.clone(),
        }
    }

    pub fn forward(&mut self, prompt: &Tensor, description: &Tensor) -> nim_error::Result<Tensor> {
        let config = &self.config;
        let device = prompt.device();
        let prompt_len = prompt.dim(1)?;
        let limit = config
            .max_position_embeddings
            .checked_sub(prompt_len)
            .filter(|limit| *limit > config.num_codebooks)
            .ok_or_else(|| nim_error::Diagnostic::message("Parler text exceeds the available context"))?;
        self.decoder.clear_kv_cache();
        self.text_encoder.clear_kv_cache();
        let encoded = self.text_encoder.forward(description)?;
        let encoded = match self.enc_to_dec_proj.as_ref() {
            Some(projection) => encoded.apply(projection)?,
            None => encoded,
        };
        let prompt = prompt.apply(&self.embed_prompts)?;
        let count = config.num_codebooks;
        let mut tokens = vec![self.decoder_start_token_id; count];
        let mut output = vec![Vec::new(); count];
        let mut sampler = LogitsProcessor::from_sampling(0, Sampling::TopK { k: 50, temperature: 1.0 });
        let mut complete = false;

        for step in 0..limit {
            let input = Tensor::from_slice(&tokens, (1, count, 1), device)?;
            let (prompt, position) = if step == 0 {
                (Some(&prompt), 0)
            } else {
                (None, step + prompt_len)
            };
            let (query, keys) = if position == 0 {
                (prompt_len + 1, prompt_len + 1)
            } else {
                (1, position + 1)
            };
            let mask = (0..query)
                .flat_map(|i| (0..keys).map(move |j| if i + keys < j + query { f32::NEG_INFINITY } else { 0.0 }))
                .collect::<Vec<_>>();
            let mask = Tensor::from_vec(mask, (query, keys), device)?;
            let logits = self.decoder.forward(&input, prompt, Some(&mask), &encoded, None, position)?;

            let logits = logits
                .iter()
                .take(step + 1)
                .map(|logits| logits.i((0, logits.dim(1)? - 1)))
                .collect::<candle_core::Result<Vec<_>>>()?;
            let logits = Tensor::stack(&logits, 0)?.to_device(&Device::Cpu)?;

            for (index, token) in tokens.iter_mut().enumerate().take(logits.dim(0)?) {
                if *token != self.pad_token_id {
                    *token = sampler.sample(&logits.i(index)?)?;
                }
            }

            if tokens.iter().all(|token| *token == self.pad_token_id) {
                complete = true;
                break;
            }

            for (index, token) in tokens.iter().copied().enumerate() {
                if token != self.decoder_start_token_id && token != self.pad_token_id {
                    output[index].push(token);
                }
            }
        }

        if !complete {
            return Err(nim_error::Diagnostic::message("Parler reached its generation limit"));
        }

        let length = output.iter().map(Vec::len).min().unwrap_or(0);

        if length == 0 {
            return Err(nim_error::Diagnostic::message("Parler produced no audio"));
        }

        for tokens in &mut output {
            tokens.truncate(length);
        }

        Ok(Tensor::new(output, &Device::Cpu)?.to_dtype(DType::I64)?.unsqueeze(0)?)
    }
}
