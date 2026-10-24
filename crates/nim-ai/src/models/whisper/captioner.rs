use std::sync::Mutex;

use candle_core::Tensor;
use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{AudioToText, Context};

use super::config::Config;
use super::model::Model;
use super::processor::Processor;

pub struct Captioner {
    model: Mutex<Model>,
    processor: Processor,
    model_config: Config,
    config: Generation,
}

#[derive(Clone)]
struct Generation {
    eos: u32,
    max_len: usize,
    prefix: Vec<u32>,
    suppress: Vec<u32>,
    begin_suppress: Vec<u32>,
}

impl Captioner {
    pub fn new(config: Config, processor: Processor, vars: VarBuilder) -> Result<Self, Error> {
        let mut forced = config.forced_decoder_ids.clone();
        forced.sort_by_key(|(index, _)| *index);
        let prefix = std::iter::once(config.decoder_start_token_id)
            .chain(forced.into_iter().map(|(_, token)| token))
            .collect();
        let generation = Generation {
            eos: config.eos_token_id,
            max_len: config.model().max_target_positions,
            prefix,
            suppress: config.model().suppress_tokens.clone(),
            begin_suppress: config.begin_suppress_tokens.clone(),
        };

        Ok(Self {
            model: Mutex::new(Model::new(&config, vars)?),
            processor,
            model_config: config,
            config: generation,
        })
    }

    fn one(&self, ctx: &Context, bytes: &[u8], beams: usize, max_len: usize) -> Result<String, Error> {
        let chunks = self.processor.tensors(bytes, &self.model_config, ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "audio-to-text model lock poisoned"))?;
        let mut captions = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let audio = model.encode(&chunk)?;
            captions.push(self.decode(ctx, &mut model, &audio, beams, max_len)?);
        }

        captions.retain(|caption| !caption.is_empty());
        Ok(captions.join(" "))
    }

    fn decode(&self, ctx: &Context, model: &mut Model, audio: &Tensor, beams: usize, max_len: usize) -> Result<String, Error> {
        let limit = max_len.min(self.config.max_len).max(self.config.prefix.len());
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

        ctx.tokenizer()?
            .decode(&tokens, true)
            .map(|text| text.trim().to_string())
            .map_err(|err| Error::custom("ai::decoding", err))
    }
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

impl AudioToText for Captioner {
    fn audio_to_text(
        &self,
        ctx: &Context,
        audio: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error> {
        audio
            .iter()
            .map(|bytes| self.one(ctx, bytes, beams.unwrap_or(1), max_len.unwrap_or(self.config.max_len)))
            .collect()
    }
}
