use std::path::Path;
use std::sync::Mutex;

use candle_core::{DType, Device, Tensor, quantized::gguf_file};
use candle_nn::VarBuilder;
use candle_transformers::models::{quantized_qwen2, qwen2};

use crate::Error;
use crate::models::{Context, Generate, GenerateOptions, Generation};
use crate::pipelines::common::decode_text;

use super::Config;

const CONTEXT_LIMIT: usize = 4096;
const MAX_LEN: usize = 128;
const PROMPT: &str = "Summarize the meaning of the user's text in concise third-person prose. \
                      Preserve only facts stated in the text. Keep short messages to one short sentence. \
                      Do not reply to the message, continue the conversation, or invent details. \
                      Return only the summary.";

pub struct Generator {
    model: Mutex<Runtime>,
    context_limit: usize,
}

enum Runtime {
    Safetensors(qwen2::ModelForCausalLM),
    Quantized(quantized_qwen2::ModelWeights),
}

impl Runtime {
    fn clear_kv_cache(&mut self) {
        match self {
            Self::Safetensors(model) => model.clear_kv_cache(),
            Self::Quantized(model) => model.clear_kv_cache(),
        }
    }

    fn forward(&mut self, input: &Tensor, offset: usize) -> Result<Tensor, Error> {
        let logits = match self {
            Self::Safetensors(model) => model.forward(input, offset)?,
            Self::Quantized(model) => model.forward(input, offset)?,
        };

        Ok(logits.flatten_all()?.to_dtype(DType::F32)?)
    }
}

impl Generator {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        Ok(Self {
            model: Mutex::new(Runtime::Safetensors(qwen2::ModelForCausalLM::new(config, vars)?)),
            context_limit: CONTEXT_LIMIT.min(config.max_position_embeddings),
        })
    }

    pub fn quantized(config: &Config, path: impl AsRef<Path>, device: &Device) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let model = quantized_qwen2::ModelWeights::from_gguf(content, &mut file, device)?;

        Ok(Self {
            model: Mutex::new(Runtime::Quantized(model)),
            context_limit: CONTEXT_LIMIT.min(config.max_position_embeddings),
        })
    }

    fn one(&self, ctx: &Context, text: &str, options: &GenerateOptions<'_>) -> Result<Generation, Error> {
        let tokenizer = ctx.tokenizer()?;
        let prompt = options.prompt.unwrap_or(PROMPT);
        let text = format!("<|im_start|>system\n{prompt}<|im_end|>\n<|im_start|>user\n{text}<|im_end|>\n<|im_start|>assistant\n");
        let encoding = tokenizer
            .encode(text, false)
            .map_err(|err| Error::custom("ai::encoding", err))?;
        let ids = encoding.get_ids();
        let max_len = options.max_len.unwrap_or(MAX_LEN);

        if ids.len().saturating_add(max_len) > self.context_limit {
            return Err(Error::custom(
                "ai::config",
                format!(
                    "Qwen prompt and output allowance exceed the {}-token context limit",
                    self.context_limit
                ),
            ));
        }

        let stops = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .map(|token| {
                tokenizer
                    .token_to_id(token)
                    .ok_or_else(|| Error::custom("ai::config", format!("Qwen tokenizer is missing {token}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "Qwen model lock poisoned"))?;

        model.clear_kv_cache();

        let result = (|| {
            let mut input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
            let mut offset = 0;
            let mut tokens = Vec::new();
            let mut sum = 0.0;

            for _ in 0..max_len {
                let logits = model.forward(&input, offset)?;
                let token = logits.argmax(0)?.to_scalar::<u32>()?;

                if stops.contains(&token) {
                    break;
                }

                let log_probs = candle_nn::ops::log_softmax(&logits, 0)?.to_vec1::<f32>()?;
                sum += log_probs[token as usize] as f64;
                tokens.push(token);
                offset += input.dim(1)?;
                input = Tensor::from_slice(&[token], (1, 1), ctx.device())?;
            }

            let confidence = if tokens.is_empty() || !sum.is_finite() {
                None
            } else {
                let value = (sum / tokens.len() as f64).exp();
                value.is_finite().then(|| value.clamp(0.0, 1.0))
            };
            let text = tokenizer
                .decode(&tokens, true)
                .map_err(|err| Error::custom("ai::decoding", err))?;

            Ok(Generation {
                text: text.trim().to_string(),
                confidence,
            })
        })();

        model.clear_kv_cache();
        result
    }
}

impl Generate for Generator {
    fn generate(&self, ctx: &Context, input: &[&[u8]], options: GenerateOptions<'_>) -> Result<Vec<Generation>, Error> {
        options.validate()?;

        if options.language.is_some() || options.beams.is_some_and(|beams| beams != 1) {
            return Err(Error::custom(
                "ai::config",
                "Qwen generation supports one beam and no language option",
            ));
        }

        let text = decode_text(input)?;
        text.iter().map(|text| self.one(ctx, text, &options)).collect()
    }
}
