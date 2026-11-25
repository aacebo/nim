mod model;

use candle_core::Tensor;

use crate::{Context, Embed, EmbedMode, EmbedOptions, models::Loader};

pub struct EmbeddingGemmaEmbedder {
    max_position_embeddings: usize,
    model: model::EmbeddingGemmaModel,
    dense: Vec<candle_nn::Linear>,
}

impl EmbeddingGemmaEmbedder {
    pub async fn new(loader: &Loader) -> nim_error::Result<Self> {
        let config: model::EmbeddingGemmaConfig = loader.config().await?;
        let vars = loader.vars().await?;
        let vars = if vars.contains_tensor("embed_tokens.weight") {
            vars.rename_f(|name: &str| name.strip_prefix("model.").unwrap_or(name).to_string())
        } else {
            vars
        };

        let mut dense = Vec::new();

        for path in ["2_Dense", "3_Dense"] {
            #[derive(serde::Deserialize)]
            struct EmbeddingGemmaConfig {
                in_features: usize,
                out_features: usize,
                bias: bool,
            }

            let config: EmbeddingGemmaConfig = loader.json(&format!("{path}/config.json")).await?;
            let file = loader
                .resolve(std::path::Path::new(&format!("{path}/model.safetensors")))
                .await?;
            let vars =
                unsafe { candle_nn::VarBuilder::from_mmaped_safetensors(&[file], candle_core::DType::F32, loader.device()) }?;
            dense.push(candle_nn::linear_b(
                config.in_features,
                config.out_features,
                config.bias,
                vars.pp("linear"),
            )?);
        }

        Ok(Self {
            max_position_embeddings: config.max_position_embeddings,
            model: model::EmbeddingGemmaModel::new(false, &config, vars)?,
            dense,
        })
    }
}

impl Embed for EmbeddingGemmaEmbedder {
    fn embed<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        Box::pin(async move { self.embed_with(ctx, input, &EmbedOptions::default()).await })
    }

    fn embed_with<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: &'a EmbedOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            options.validate()?;
            let dimensions = options.dimensions.unwrap_or(768);

            if ![128, 256, 512, 768].contains(&dimensions) {
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
                        "task: {} | query: {text}",
                        options.instruction.as_deref().unwrap_or("search result")
                    ),
                    EmbedMode::Document => format!("title: none | text: {text}"),
                };

                let encoding = tokenizer.encode(text, true).map_err(nim_error::Diagnostic::message)?;
                let length = encoding.get_ids().len();

                if length == 0 {
                    return Err(nim_error::Diagnostic::message("embedding input has no tokens"));
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
                    return Err(nim_error::Diagnostic::message("invalid embedding norm"));
                }

                for value in &mut vector {
                    *value /= norm;
                }

                output.push(vector);
            }

            Ok(output)
        })
    }
}
