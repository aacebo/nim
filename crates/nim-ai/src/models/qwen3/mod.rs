use candle_core::Tensor;

use crate::Error;
use crate::models::{Context, Embed, EmbedMode, EmbedOptions, Loader};

pub struct Embedder {
    model: candle_transformers::models::qwen3::Model,
}

impl Embedder {
    pub fn new(loader: &Loader) -> Result<Self, Error> {
        let config = loader.config()?;
        let vars = loader.vars()?;
        let vars = if vars.contains_tensor("embed_tokens.weight") {
            vars.rename_f(|name: &str| name.strip_prefix("model.").unwrap_or(name).to_string())
        } else {
            vars
        };

        Ok(Self {
            model: candle_transformers::models::qwen3::Model::new(&config, vars)?,
        })
    }
}

impl Embed for Embedder {
    fn embed(&self, ctx: &Context, input: &[&[u8]]) -> Result<Vec<Vec<f32>>, Error> {
        self.embed_with(ctx, input, &EmbedOptions::default())
    }

    fn embed_with(&self, ctx: &Context, input: &[&[u8]], options: &EmbedOptions) -> Result<Vec<Vec<f32>>, Error> {
        options.validate()?;
        let dimensions = options.dimensions.unwrap_or(1024);

        if !(32..=1024).contains(&dimensions) {
            return Err(Error::custom("ai::config", "unsupported embedding dimensions"));
        }

        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(|err| Error::custom("ai::encoding", err)))
            .collect::<Result<Vec<_>, _>>()?;
        let mut tokenizer = ctx.tokenizer()?.clone();
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(Some(tokenizers::TruncationParams {
                max_length: 32768,
                ..Default::default()
            }))
            .map_err(|err| Error::custom("ai::encoding", err))?;
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

            let encoding = tokenizer
                .encode(text, true)
                .map_err(|err| Error::custom("ai::encoding", err))?;
            let length = encoding.get_ids().len();

            if length == 0 {
                return Err(Error::custom("ai::encoding", "embedding input has no tokens"));
            }

            let ids = Tensor::new(encoding.get_ids(), ctx.device())?.unsqueeze(0)?;
            let mut model = self.model.clone();
            let hidden = model.forward(&ids, 0)?;
            let vector = hidden.narrow(1, length - 1, 1)?;
            let mut vector = vector.flatten_all()?.to_vec1::<f32>()?;
            vector.truncate(dimensions);
            let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();

            if !norm.is_finite() || norm == 0.0 {
                return Err(Error::custom("ai::invalid", "invalid embedding norm"));
            }

            for value in &mut vector {
                *value /= norm;
            }

            output.push(vector);
        }

        Ok(output)
    }
}
