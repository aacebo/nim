use std::sync::Mutex;

use candle_core::Tensor;
use candle_nn::VarBuilder;

use crate::types::TextOutput;
use crate::{Caption, CaptionOptions, Context};

use super::config::WhisperConfig;
use super::model::WhisperModel;
use super::processor::WhisperProcessor;

pub struct WhisperCaptioner {
    model: Mutex<WhisperModel>,
    processor: WhisperProcessor,
    model_config: WhisperConfig,
    config: WhisperGenerationConfig,
}

impl WhisperCaptioner {
    pub fn new(config: WhisperConfig, processor: WhisperProcessor, vars: VarBuilder) -> nim_error::Result<Self> {
        let mut forced = config.forced_decoder_ids.clone();
        forced.sort_by_key(|(index, _)| *index);
        let prefix = std::iter::once(config.decoder_start_token_id)
            .chain(forced.into_iter().map(|(_, token)| token))
            .collect();
        let decoding = WhisperGenerationConfig {
            eos: config.eos_token_id,
            max_positions: config.model().max_target_positions,
            prefix,
            suppress: config.model().suppress_tokens.clone(),
            begin_suppress: config.begin_suppress_tokens.clone(),
        };

        Ok(Self {
            model: Mutex::new(WhisperModel::new(&config, vars)?),
            processor,
            model_config: config,
            config: decoding,
        })
    }

    fn one(&self, ctx: &Context, bytes: &[u8], beams: usize, max_tokens: usize) -> nim_error::Result<String> {
        let chunks = self.processor.tensors(bytes, &self.model_config, ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("audio-to-text model lock poisoned"))?;
        let mut captions = Vec::with_capacity(chunks.len());
        let mut remaining = max_tokens;

        for chunk in chunks {
            if remaining == 0 {
                break;
            }

            let audio = model.encode(&chunk)?;
            let (caption, used) = self.decode(ctx, &mut model, &audio, beams, remaining)?;
            remaining -= used;
            captions.push(caption);
        }

        captions.retain(|caption| !caption.is_empty());
        Ok(captions.join(" "))
    }

    fn decode(
        &self,
        ctx: &Context,
        model: &mut WhisperModel,
        audio: &Tensor,
        beams: usize,
        max_tokens: usize,
    ) -> nim_error::Result<(String, usize)> {
        let limit = self.config.prefix.len()
            + crate::models::output_limit(
                "Whisper",
                self.config.max_positions,
                self.config.prefix.len(),
                Some(max_tokens),
            )?;
        let mut live = vec![(0f64, self.config.prefix.clone())];
        let mut done = Vec::new();

        while live.iter().any(|(_, tokens)| tokens.len() < limit) {
            let mut candidates = Vec::new();

            for (score, tokens) in &live {
                let mut probabilities = candle_nn::ops::log_softmax(&model.logits(audio, tokens)?, 0)?.to_vec1::<f32>()?;

                for token in &self.config.suppress {
                    probabilities[*token as usize] = f32::NEG_INFINITY;
                }

                if tokens.len() == self.config.prefix.len() {
                    for token in &self.config.begin_suppress {
                        probabilities[*token as usize] = f32::NEG_INFINITY;
                    }
                }

                let mut next: Vec<_> = probabilities.iter().enumerate().collect();
                next.sort_by(|(_, left), (_, right)| right.total_cmp(left));

                for (token, probability) in next
                    .into_iter()
                    .filter(|(token, _)| !repeats(tokens, *token as u32))
                    .take(beams.max(1))
                {
                    candidates.push((*score + *probability as f64, tokens.clone(), token as u32));
                }
            }

            candidates.sort_by(|left, right| right.0.total_cmp(&left.0));
            live.clear();

            for (score, mut tokens, token) in candidates {
                tokens.push(token);

                if token == self.config.eos || tokens.len() >= limit {
                    done.push((score, tokens));
                } else if live.len() < beams.max(1) {
                    live.push((score, tokens));
                }

                if live.len() >= beams.max(1) && done.len() >= beams.max(1) {
                    break;
                }
            }
        }

        done.extend(live);
        done.sort_by(|left, right| right.0.total_cmp(&left.0));
        let tokens = done.into_iter().next().map(|(_, tokens)| tokens).unwrap_or_default();
        let used = tokens.len().saturating_sub(self.config.prefix.len());
        let text = ctx
            .tokenizer()?
            .decode(&tokens, true)
            .map_err(nim_error::Diagnostic::message)?;

        Ok((text.trim().to_string(), used))
    }
}

impl Caption for WhisperCaptioner {
    fn caption<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: CaptionOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            options.validate()?;

            let beams = options.beams;
            let max_tokens = options.max_tokens;
            input
                .iter()
                .map(|bytes| {
                    self.one(
                        ctx,
                        bytes,
                        beams.unwrap_or(1),
                        max_tokens.unwrap_or(crate::models::DEFAULT_MAX_TOKENS),
                    )
                })
                .map(|result| result.map(|text| TextOutput { text, confidence: None }))
                .collect()
        })
    }
}

#[derive(Clone)]
struct WhisperGenerationConfig {
    eos: u32,
    max_positions: usize,
    prefix: Vec<u32>,
    suppress: Vec<u32>,
    begin_suppress: Vec<u32>,
}

fn repeats(tokens: &[u32], token: u32) -> bool {
    const NGRAM: usize = 3;

    if tokens.len() < NGRAM - 1 {
        return false;
    }

    let start = tokens.len() + 1 - NGRAM;
    let candidate = &tokens[start..];

    tokens[..start]
        .windows(NGRAM)
        .any(|ngram| ngram[..NGRAM - 1] == candidate[..NGRAM - 1] && ngram[NGRAM - 1] == token)
}
