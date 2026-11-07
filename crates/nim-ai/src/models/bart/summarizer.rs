use std::sync::Mutex;

use candle_core::Tensor;
use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{Context, Generate, GenerateOptions, Generation};

use super::config::Config;
use super::generate;
use super::model::Bart;

/// BART behind the `Generate` capability.
///
/// `Bart` itself cannot implement the trait: generation mutates its KV cache, so it needs
/// `&mut self` while the trait -- being cached as `Arc<dyn Generate>` and shared across threads --
/// hands out `&self`. The mutex is what reconciles the two, and it lives here rather than in the
/// model so the architecture stays a plain candle module.
pub struct Summarizer {
    model: Mutex<Bart>,
    generation: generate::Config,
    max_position_embeddings: usize,
}

impl Summarizer {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        Ok(Self {
            model: Mutex::new(Bart::new(config, vars)?),
            generation: generate::Config::from(config),
            max_position_embeddings: config.max_position_embeddings,
        })
    }

    pub fn beams(mut self, beams: usize) -> Self {
        self.generation = self.generation.beams(beams);
        self
    }
}

impl Generate for Summarizer {
    fn generate(&self, ctx: &Context, input: &[&[u8]], options: GenerateOptions<'_>) -> Result<Vec<Generation>, Error> {
        options.validate()?;

        if options.prompt.is_some() || options.language.is_some() {
            return Err(Error::custom(
                "ai::config",
                "BART summarization does not support prompt or language",
            ));
        }

        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(|err| Error::custom("ai::encoding", err)))
            .collect::<Result<Vec<_>, _>>()?;
        let mut generation = self.generation;

        if let Some(beams) = options.beams {
            generation = generation.beams(beams);
        }
        if let Some(max_len) = options.max_len {
            generation = generation.max_length(max_len);
        }

        text.iter().map(|text| self.one(ctx, text, &generation)).collect()
    }
}

impl Summarizer {
    fn one(&self, ctx: &Context, text: &str, generation: &generate::Config) -> Result<Generation, Error> {
        let encoding = ctx.encode_one(text)?;
        let ids = encoding.get_ids();

        // The learned positional embeddings only cover `max_position_embeddings`.
        let ids = &ids[..ids.len().min(self.max_position_embeddings)];
        let input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "summarization model lock poisoned"))?;

        let (tokens, confidence) = generate::run(&mut model, generation, &input, ctx.device())?;

        ctx.tokenizer()?
            .decode(&tokens, true)
            .map(|summary| Generation {
                text: summary.trim().to_string(),
                confidence,
            })
            .map_err(|err| Error::custom("ai::decoding", err))
    }
}
