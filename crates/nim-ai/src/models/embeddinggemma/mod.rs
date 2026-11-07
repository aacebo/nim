mod model;

use candle_core::Tensor;

use crate::Error;
use crate::models::{Context, Embed, EmbedMode, EmbedOptions, Loader};

pub struct Embedder {
    model: model::Model,
    dense: Vec<candle_nn::Linear>,
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
            model: model::Model::new(false, &config, vars)?,
            dense: ["2_Dense", "3_Dense"]
                .iter()
                .map(|path| {
                    #[derive(serde::Deserialize)]
                    struct Config {
                        in_features: usize,
                        out_features: usize,
                        bias: bool,
                    }

                    let config: Config = loader.json(&format!("{path}/config.json"))?;
                    let file = loader.resolve(std::path::Path::new(&format!("{path}/model.safetensors")))?;
                    let vars = unsafe {
                        candle_nn::VarBuilder::from_mmaped_safetensors(&[file], candle_core::DType::F32, loader.device())
                    }?;

                    Ok(candle_nn::linear_b(
                        config.in_features,
                        config.out_features,
                        config.bias,
                        vars.pp("linear"),
                    )?)
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}

impl Embed for Embedder {
    fn embed(&self, ctx: &Context, input: &[&[u8]]) -> Result<Vec<Vec<f32>>, Error> {
        self.embed_with(ctx, input, &EmbedOptions::default())
    }

    fn embed_with(&self, ctx: &Context, input: &[&[u8]], options: &EmbedOptions) -> Result<Vec<Vec<f32>>, Error> {
        options.validate()?;
        let dimensions = options.dimensions.unwrap_or(768);

        if ![128, 256, 512, 768].contains(&dimensions) {
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
                max_length: 2048,
                ..Default::default()
            }))
            .map_err(|err| Error::custom("ai::encoding", err))?;

        let mut output = Vec::with_capacity(text.len());

        for text in text {
            let text = match options.mode {
                EmbedMode::Raw => text.to_string(),
                EmbedMode::Query => format!(
                    "task: {} | query: {text}",
                    options.instruction.as_deref().unwrap_or("search result")
                ),
                EmbedMode::Document => format!("title: none | text: {text}"),
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
            let vector = hidden.mean(1)?;
            let mut vector = vector;

            for dense in &self.dense {
                vector = vector.apply(dense)?;
            }

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
