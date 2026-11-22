mod generator;
mod template;

use candle_core::Tensor;

use crate::models::{Context, Embed, EmbedMode, EmbedOptions, Loader};

pub use candle_transformers::models::qwen3::Config;
pub use generator::Qwen3Generator;
pub use template::Qwen3Template;

pub const MODEL: &str = "Qwen/Qwen3-32B";
pub const QUANTIZED_MODEL: &str = "Qwen/Qwen3-32B-GGUF";
pub const WEIGHTS: &str = "Qwen3-32B-Q4_K_M.gguf";
pub const COMPLETION_MODEL: &str = "Qwen/Qwen3-8B";
pub const COMPLETION_QUANTIZED_MODEL: &str = "Qwen/Qwen3-8B-GGUF";
pub const COMPLETION_WEIGHTS: &str = "Qwen3-8B-Q4_K_M.gguf";

pub struct Qwen3Embedder {
    max_position_embeddings: usize,
    model: candle_transformers::models::qwen3::Model,
}

impl Qwen3Embedder {
    pub fn new(loader: &Loader) -> nim_error::Result<Self> {
        let config: Config = loader.config()?;
        let vars = loader.vars()?;
        let vars = if vars.contains_tensor("embed_tokens.weight") {
            vars.rename_f(|name: &str| name.strip_prefix("model.").unwrap_or(name).to_string())
        } else {
            vars
        };

        Ok(Self {
            max_position_embeddings: config.max_position_embeddings,
            model: candle_transformers::models::qwen3::Model::new(&config, vars)?,
        })
    }
}

impl Embed for Qwen3Embedder {
    fn embed(&self, ctx: &Context, input: &[&[u8]]) -> nim_error::Result<Vec<Vec<f32>>> {
        self.embed_with(ctx, input, &EmbedOptions::default())
    }

    fn embed_with(&self, ctx: &Context, input: &[&[u8]], options: &EmbedOptions) -> nim_error::Result<Vec<Vec<f32>>> {
        options.validate()?;
        let dimensions = options.dimensions.unwrap_or(1024);

        if !(32..=1024).contains(&dimensions) {
            return Err(nim_error::Diagnostic::message("unsupported embedding dimensions"));
        }

        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        let mut tokenizer = ctx.tokenizer()?.clone();
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(Some(tokenizers::TruncationParams {
                max_length: self.max_position_embeddings,
                ..Default::default()
            }))
            .map_err(nim_error::Diagnostic::message)?;
        let mut output = Vec::with_capacity(text.len());

        for text in text {
            let text = match options.mode {
                EmbedMode::Raw => text.to_string(),
                EmbedMode::Query => format!(
                    "Instruct: {}\nQuery: {text}",
                    options
                        .instruction
                        .as_deref()
                        .unwrap_or("Given a web search query, retrieve relevant passages that answer the query")
                ),
                EmbedMode::Document => text.to_string(),
            };

            let encoding = tokenizer.encode(text, true).map_err(nim_error::Diagnostic::message)?;
            let length = encoding.get_ids().len();

            if length == 0 {
                return Err(nim_error::Diagnostic::message("embedding input has no tokens"));
            }

            let ids = Tensor::new(encoding.get_ids(), ctx.device())?.unsqueeze(0)?;
            let mut model = self.model.clone();
            let hidden = model.forward(&ids, 0)?;
            let vector = hidden.narrow(1, length - 1, 1)?;
            let mut vector = vector.flatten_all()?.to_vec1::<f32>()?;
            vector.truncate(dimensions);
            let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();

            if !norm.is_finite() || norm == 0.0 {
                return Err(nim_error::Diagnostic::message("invalid embedding norm"));
            }

            for value in &mut vector {
                *value /= norm;
            }

            output.push(vector);
        }

        Ok(output)
    }
}
