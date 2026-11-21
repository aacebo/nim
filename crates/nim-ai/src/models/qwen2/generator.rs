use std::path::Path;
use std::sync::Mutex;

use candle_core::{DType, Device, Tensor, quantized::gguf_file};
use candle_nn::VarBuilder;
use candle_transformers::models::{quantized_qwen2, qwen2};

use crate::{Chat, ChatAssistantMessage, ChatOptions, Context, Summarize, SummarizeOptions, types::TextOutput};

use super::Config;

pub struct Generator {
    model: Mutex<Runtime>,
    context_limit: usize,
    template: hf_chat_template::ChatTemplate,
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
    pub fn new(config: &Config, vars: VarBuilder, template: hf_chat_template::ChatTemplate) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Runtime::Safetensors(qwen2::ModelForCausalLM::new(config, vars)?)),
            context_limit: config.max_position_embeddings,
            template,
        })
    }

    pub fn quantized(
        config: &Config,
        path: impl AsRef<Path>,
        device: &Device,
        template: hf_chat_template::ChatTemplate,
    ) -> nim_error::Result<Self> {
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
            template,
        })
    }

    fn one(&self, ctx: &Context, options: ChatOptions) -> nim_error::Result<ChatAssistantMessage> {
        let tokenizer = ctx.tokenizer()?;
        let compat = options.compat;
        let max_len = options.max_len;
        let input: hf_chat_template::RenderInput = options.try_into()?;
        let text = self.template.render(&input).map_err(nim_error::Diagnostic::message)?;
        let encoding = tokenizer.encode(text, false).map_err(nim_error::Diagnostic::message)?;
        let ids = encoding.get_ids();
        let max_len = crate::models::output_limit("Qwen", self.context_limit, ids.len(), max_len)?;
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

            let _ = confidence;
            ChatAssistantMessage::from_qwen(text, compat)
        })();

        model.clear_kv_cache();
        result
    }
}

impl Chat for Generator {
    fn chat(&self, ctx: &Context, options: ChatOptions) -> nim_error::Result<ChatAssistantMessage> {
        options.validate()?;

        if options.beams.is_some_and(|beams| beams != 1) {
            return Err(nim_error::Diagnostic::message("Qwen chat supports one beam"));
        }

        self.one(ctx, options)
    }
}

impl Summarize for Generator {
    fn summarize(&self, ctx: &Context, input: &[&[u8]], options: SummarizeOptions) -> nim_error::Result<Vec<TextOutput>> {
        options.validate()?;

        let responses = input.iter().map(|bytes| {
            let text = std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from)?;
            self.chat(ctx, ChatOptions {
                messages: vec![crate::ChatUserMessage { content: crate::ChatMessageContent::Text(format!("Summarize the text concisely and accurately, using only information present in the text. Return only the summary.\n\n{text}")) }.into()],
            beams: options.beams,
            max_len: options.max_len,
                ..Default::default()
            })
        }).collect::<nim_error::Result<Vec<_>>>()?;

        Ok(responses
            .into_iter()
            .map(|response| TextOutput {
                text: match response.content {
                    Some(crate::ChatMessageContent::Text(text)) => text,
                    _ => String::new(),
                },
                confidence: None,
            })
            .collect())
    }
}
