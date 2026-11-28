use std::path::Path;
use std::sync::Mutex;

use candle_core::{DType, Device, Tensor, quantized::gguf_file};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen3::Config;
use candle_transformers::models::{quantized_qwen3, qwen3};
use nim_chat::{
    Chat, ChatAgentMessage, ChatMessage, ChatMessageContent, ChatUserMessage, Format, Parser, format::chatml::ChatML,
};

use crate::types::TextOutput;
use crate::{Complete, Context, Summarize, SummarizeOptions, models::qwen::QwenSampler};

pub struct Qwen3Generator {
    model: Mutex<Qwen3Runtime>,
    context_limit: usize,
}

enum Qwen3Runtime {
    Safetensors(qwen3::ModelForCausalLM),
    Quantized(quantized_qwen3::ModelWeights),
}

impl Qwen3Runtime {
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

impl Qwen3Generator {
    pub fn new(config: &Config, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Qwen3Runtime::Safetensors(qwen3::ModelForCausalLM::new(config, vars)?)),
            context_limit: config.max_position_embeddings,
        })
    }

    pub fn quantized(config: &Config, path: impl AsRef<Path>, device: &Device) -> nim_error::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let content = gguf_file::Content::read(&mut file)?;
        let context_limit = content
            .metadata
            .get("qwen3.context_length")
            .ok_or_else(|| nim_error::Diagnostic::message("GGUF is missing qwen3.context_length"))?
            .to_u32()? as usize;
        let model = quantized_qwen3::ModelWeights::from_gguf(content, &mut file, device)?;

        Ok(Self {
            model: Mutex::new(Qwen3Runtime::Quantized(model)),
            context_limit: config.max_position_embeddings.min(context_limit),
        })
    }

    async fn one(&self, ctx: &Context<'_>, chat: Chat) -> nim_error::Result<ChatAgentMessage> {
        let tokenizer = ctx.tokenizer()?;
        let max_tokens = chat.max_tokens;
        let sampling_thinking = chat.thinking;
        let temperature = chat.temperature.unwrap_or(if sampling_thinking { 0.6 } else { 0.7 });
        let top_p = chat.top_p.unwrap_or(if sampling_thinking { 0.95 } else { 0.8 });
        let respond = chat
            .messages
            .last()
            .is_none_or(|message| !matches!(message, ChatMessage::Agent(_)));
        let thinking = chat.thinking && respond;
        let mut text = ChatML.render(&chat)?;

        if respond {
            if thinking {
                text.push_str("<think>\n");
            } else {
                text.push_str("<think>\n\n</think>\n\n");
            }
        }

        let encoding = tokenizer.encode(text, false).map_err(nim_error::Diagnostic::message)?;
        let ids = encoding.get_ids();
        let max_tokens = crate::models::output_limit("Qwen3", self.context_limit, ids.len(), max_tokens)?;
        let stops = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .map(|token| {
                tokenizer
                    .token_to_id(token)
                    .ok_or_else(|| nim_error::Diagnostic::message(format!("Qwen3 tokenizer is missing {token}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result: nim_error::Result<String> = {
            let mut model = self
                .model
                .lock()
                .map_err(|_| nim_error::Diagnostic::message("Qwen3 model lock poisoned"))?;

            model.clear_kv_cache();

            let result = (|| {
                let mut input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
                let mut offset = 0;
                let mut tokens = Vec::new();
                let mut sampler = QwenSampler::new(temperature, top_p);

                for _ in 0..max_tokens {
                    let logits = model.forward(&input, offset)?;
                    let token = sampler.sample(&logits)?;

                    if stops.contains(&token) {
                        break;
                    }

                    tokens.push(token);
                    offset += input.dim(1)?;
                    input = Tensor::from_slice(&[token], (1, 1), ctx.device())?;
                }

                let text = tokenizer.decode(&tokens, false).map_err(nim_error::Diagnostic::message)?;

                Ok(text)
            })();

            model.clear_kv_cache();
            result
        };

        let mut text = result?;

        if thinking {
            text.insert_str(0, "<think>\n");

            if !text.contains("</think>") {
                text.push_str("</think>");
            }
        }

        nim_chat::format::chatml::parse_agent_message(&mut Parser::from_src(&text))
    }
}

impl Complete for Qwen3Generator {
    fn complete<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        chat: Chat,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<ChatAgentMessage>> {
        Box::pin(async move {
            chat.validate()?;

            if chat.beams.is_some_and(|beams| beams != 1) {
                return Err(nim_error::Diagnostic::message("Qwen3 completion supports one beam"));
            }

            self.one(ctx, chat).await
        })
    }
}

impl Summarize for Qwen3Generator {
    fn summarize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: SummarizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            options.validate()?;

            let mut output = Vec::with_capacity(input.len());

            for bytes in input {
                let text = std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from)?;
                let prompt = concat!(
                    "Summarize the text concisely and accurately, using only information present in the text. ",
                    "Return only the summary.",
                );
                let chat = Chat {
                    messages: vec![
                        ChatUserMessage {
                            name: None,
                            content: ChatMessageContent::String(format!("{prompt}\n\n{text}")),
                            extra: Default::default(),
                        }
                        .into(),
                    ],
                    beams: options.beams,
                    max_tokens: options.max_tokens,
                    ..Default::default()
                };

                let response = self.complete(ctx, chat).await?;

                output.push(TextOutput {
                    text: crate::tasks::response_text(response.content),
                    confidence: None,
                });
            }

            Ok(output)
        })
    }
}
