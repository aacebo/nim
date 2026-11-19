use crate::types::TextOutput;
use std::path::Path;
use std::sync::Mutex;

use candle_core::{DType, Device, Tensor, quantized::gguf_file};
use candle_nn::VarBuilder;
use candle_transformers::models::{quantized_qwen2, qwen2};

use crate::models::{Chat, ChatOptions, Context, Summarize, SummarizeOptions};

use super::Config;

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

    fn forward(&mut self, input: &Tensor, offset: usize) -> nim_error::Result<Tensor> {
        let logits = match self {
            Self::Safetensors(model) => model.forward(input, offset)?,
            Self::Quantized(model) => model.forward(input, offset)?,
        };

        Ok(logits.flatten_all()?.to_dtype(DType::F32)?)
    }
}

impl Generator {
    pub fn new(config: &Config, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Runtime::Safetensors(qwen2::ModelForCausalLM::new(config, vars)?)),
            context_limit: config.max_position_embeddings,
        })
    }

    pub fn quantized(config: &Config, path: impl AsRef<Path>, device: &Device) -> nim_error::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let context_limit = content
            .metadata
            .get("qwen2.context_length")
            .ok_or_else(|| nim_error::Diagnostic::message("GGUF is missing qwen2.context_length"))?
            .to_u32()? as usize;
        let model = quantized_qwen2::ModelWeights::from_gguf(content, &mut file, device)?;

        Ok(Self {
            model: Mutex::new(Runtime::Quantized(model)),
            context_limit: config.max_position_embeddings.min(context_limit),
        })
    }

    fn one(&self, ctx: &Context, text: &str, options: &ChatOptions<'_>) -> nim_error::Result<TextOutput> {
        let tokenizer = ctx.tokenizer()?;
        let system = options
            .prompt
            .map(|prompt| format!("<|im_start|>system\n{prompt}<|im_end|>\n"));
        let text = format!(
            "{}<|im_start|>user\n{text}<|im_end|>\n<|im_start|>assistant\n",
            system.unwrap_or_default()
        );
        let encoding = tokenizer.encode(text, false).map_err(nim_error::Diagnostic::message)?;
        let ids = encoding.get_ids();
        let max_len = crate::models::output_limit("Qwen", self.context_limit, ids.len(), options.max_len)?;
        let stops = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .map(|token| {
                tokenizer
                    .token_to_id(token)
                    .ok_or_else(|| nim_error::Diagnostic::message(format!("Qwen tokenizer is missing {token}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("Qwen model lock poisoned"))?;

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

            let text = tokenizer.decode(&tokens, true).map_err(nim_error::Diagnostic::message)?;

            Ok(TextOutput {
                text: text.trim().to_string(),
                confidence,
            })
        })();

        model.clear_kv_cache();
        result
    }
}

impl Chat for Generator {
    fn chat(&self, ctx: &Context, input: &[&[u8]], options: ChatOptions<'_>) -> nim_error::Result<Vec<TextOutput>> {
        options.validate()?;

        if options.beams.is_some_and(|beams| beams != 1) {
            return Err(nim_error::Diagnostic::message("Qwen chat supports one beam"));
        }

        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        text.iter().map(|text| self.one(ctx, text, &options)).collect()
    }
}

impl Summarize for Generator {
    fn summarize(&self, ctx: &Context, input: &[&[u8]], options: SummarizeOptions) -> nim_error::Result<Vec<TextOutput>> {
        options.validate()?;

        let responses = self.chat(ctx, input, ChatOptions {
            prompt: Some("Summarize the text concisely and accurately, using only information present in the text. Return only the summary."),
            beams: options.beams,
            max_len: options.max_len,
        })?;

        Ok(responses)
    }
}
