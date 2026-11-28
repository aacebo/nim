use std::path::Path;
use std::sync::Mutex;

use candle_core::{DType, Device, Tensor, quantized::gguf_file};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::Config;
use candle_transformers::models::{quantized_qwen2, qwen2};
use candle_transformers::utils::apply_repeat_penalty;
use nim_chat::{
    Chat, ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageRole, ChatSystemMessage, ChatUserMessage, Format, Parser,
    format::chatml::ChatML,
};

use crate::{Complete, Context, Summarize, SummarizeOptions, models::qwen::QwenSampler, types::TextOutput};

pub struct Qwen2Generator {
    model: Mutex<Qwen2Runtime>,
    context_limit: usize,
}

enum Qwen2Runtime {
    Safetensors(qwen2::ModelForCausalLM),
    Quantized(quantized_qwen2::ModelWeights),
}

impl Qwen2Runtime {
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

impl Qwen2Generator {
    pub fn new(config: &Config, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Qwen2Runtime::Safetensors(qwen2::ModelForCausalLM::new(config, vars)?)),
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
            model: Mutex::new(Qwen2Runtime::Quantized(model)),
            context_limit: config.max_position_embeddings.min(context_limit),
        })
    }

    async fn one(&self, ctx: &Context<'_>, mut chat: Chat) -> nim_error::Result<ChatAgentMessage> {
        let tokenizer = ctx.tokenizer()?;
        let max_tokens = chat.max_tokens;
        let temperature = chat.temperature.unwrap_or(0.7);
        let top_p = chat.top_p.unwrap_or(0.8);

        if chat.thinking {
            let instruction = "Before your answer, reason through the request inside <think> and </think> tags. Then provide your answer after the closing tag.";

            if let Some(ChatMessage::System(message)) = chat
                .messages
                .iter_mut()
                .find(|message| matches!(message, ChatMessage::System(value) if value.role == ChatMessageRole::System))
            {
                message.content.push(format!("\n\n{instruction}"));
            } else {
                chat.messages.insert(
                    0,
                    ChatSystemMessage {
                        role: ChatMessageRole::System,
                        name: None,
                        content: ChatMessageContent::String(instruction.into()),
                        extra: Default::default(),
                    }
                    .into(),
                );
            }
        }

        let text = ChatML.render(&chat)?;
        let encoding = tokenizer.encode(text, false).map_err(nim_error::Diagnostic::message)?;
        let ids = encoding.get_ids();
        let max_tokens = crate::models::output_limit("Qwen", self.context_limit, ids.len(), max_tokens)?;
        let stops = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .map(|token| {
                tokenizer
                    .token_to_id(token)
                    .ok_or_else(|| nim_error::Diagnostic::message(format!("Qwen tokenizer is missing {token}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result: nim_error::Result<String> = {
            let mut model = self
                .model
                .lock()
                .map_err(|_| nim_error::Diagnostic::message("Qwen model lock poisoned"))?;

            model.clear_kv_cache();

            let result = (|| {
                let mut input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
                let mut offset = 0;
                let mut tokens = Vec::new();
                let mut history = ids.to_vec();
                let mut sampler = QwenSampler::new(temperature, top_p);

                for _ in 0..max_tokens {
                    let logits = model.forward(&input, offset)?;
                    let logits = apply_repeat_penalty(&logits, 1.1, &history)?;
                    let token = sampler.sample(&logits)?;

                    if stops.contains(&token) {
                        break;
                    }

                    tokens.push(token);
                    history.push(token);
                    offset += input.dim(1)?;
                    input = Tensor::from_slice(&[token], (1, 1), ctx.device())?;
                }

                let text = tokenizer.decode(&tokens, false).map_err(nim_error::Diagnostic::message)?;
                Ok(text)
            })();

            model.clear_kv_cache();
            result
        };

        let text = result?;
        nim_chat::format::chatml::parse_agent_message(&mut Parser::from_src(&text))
    }
}

impl Complete for Qwen2Generator {
    fn complete<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        chat: Chat,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<ChatAgentMessage>> {
        Box::pin(async move {
            chat.validate()?;

            if chat.beams.is_some_and(|beams| beams != 1) {
                return Err(nim_error::Diagnostic::message("Qwen completion supports one beam"));
            }

            self.one(ctx, chat).await
        })
    }
}

impl Summarize for Qwen2Generator {
    fn summarize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: SummarizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            options.validate()?;

            let mut responses = Vec::with_capacity(input.len());

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

                responses.push(self.complete(ctx, chat).await?);
            }

            Ok(responses
                .into_iter()
                .map(|response| TextOutput {
                    text: crate::tasks::response_text(response.content),
                    confidence: None,
                })
                .collect())
        })
    }
}
