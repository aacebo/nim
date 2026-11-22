use crate::types::TextOutput;
use std::sync::Mutex;

use candle_core::Tensor;
use candle_nn::VarBuilder;

use crate::models::{Context, Summarize, SummarizeOptions};

use super::config::BartConfig;
use super::generate;
use super::model::Bart;

/// BART behind the `Summarize` capability.
///
/// `Bart` itself cannot implement the trait: decoding mutates its KV cache, so it needs
/// `&mut self` while the trait -- being cached as `Arc<dyn Summarize>` and shared across threads --
/// hands out `&self`. The mutex is what reconciles the two, and it lives here rather than in the
/// model so the architecture stays a plain candle module.
pub struct BartSummarizer {
    model: Mutex<Bart>,
    decoding: generate::BartGenerationConfig,
    max_position_embeddings: usize,
}

impl BartSummarizer {
    pub fn new(config: &BartConfig, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Bart::new(config, vars)?),
            decoding: generate::BartGenerationConfig::from(config),
            max_position_embeddings: config.max_position_embeddings,
        })
    }

    pub fn beams(mut self, beams: usize) -> Self {
        self.decoding = self.decoding.beams(beams);
        self
    }
}

impl Summarize for BartSummarizer {
    fn summarize(&self, ctx: &Context, input: &[&[u8]], options: SummarizeOptions) -> nim_error::Result<Vec<TextOutput>> {
        options.validate()?;

        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        let mut decoding = self.decoding;

        if let Some(beams) = options.beams {
            decoding = decoding.beams(beams);
        }

        let max_len = options.max_len.unwrap_or(self.max_position_embeddings);

        if max_len > self.max_position_embeddings {
            return Err(nim_error::Diagnostic::message(
                "summary length exceeds model position embeddings",
            ));
        }

        decoding = decoding.max_length(max_len);
        text.iter().map(|text| self.one(ctx, text, &decoding)).collect()
    }
}

impl BartSummarizer {
    fn one(&self, ctx: &Context, text: &str, decoding: &generate::BartGenerationConfig) -> nim_error::Result<TextOutput> {
        let encoding = ctx.encode_one(text)?;
        let ids = encoding.get_ids();

        // The learned positional embeddings only cover `max_position_embeddings`.
        let ids = &ids[..ids.len().min(self.max_position_embeddings)];
        let input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("summarization model lock poisoned"))?;

        let (tokens, confidence) = generate::run(&mut model, decoding, &input, ctx.device())?;

        ctx.tokenizer()?
            .decode(&tokens, true)
            .map(|summary| TextOutput {
                text: summary.trim().to_string(),
                confidence,
            })
            .map_err(nim_error::Diagnostic::message)
    }
}
