use std::sync::Mutex;

use candle_core::Tensor;
use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{Context, Transcribe};

use super::config::Config;
use super::model::Model;
use super::processor::Processor;

const FIRST_LANGUAGE_TOKEN: u32 = 50_259;
const LAST_LANGUAGE_TOKEN: u32 = 50_357;

pub struct Transcriber {
    model: Mutex<Model>,
    processor: Processor,
    model_config: Config,
    config: Generation,
}

#[derive(Clone)]
struct Generation {
    start: u32,
    eos: u32,
    max_len: usize,
    suppress: Vec<u32>,
    begin_suppress: Vec<u32>,
}

impl Transcriber {
    pub fn new(config: Config, processor: Processor, vars: VarBuilder) -> Result<Self, Error> {
        let generation = Generation {
            start: config.decoder_start_token_id,
            eos: config.eos_token_id,
            max_len: config.model().max_target_positions,
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

    fn one(&self, ctx: &Context, bytes: &[u8], language: Option<&str>, beams: usize, max_len: usize) -> Result<String, Error> {
        let chunks = self.processor.tensors(bytes, &self.model_config, ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "transcription model lock poisoned"))?;
        let first = model.encode(&chunks[0])?;
        let language = match language {
            Some(language) => language_token(ctx, language)?,
            None => self.detect_language(&mut model, &first)?,
        };
        let mut transcripts = vec![self.decode(ctx, &mut model, &first, language, beams, max_len)?];

        for chunk in chunks.iter().skip(1) {
            let audio = model.encode(chunk)?;
            transcripts.push(self.decode(ctx, &mut model, &audio, language, beams, max_len)?);
        }

        transcripts.retain(|transcript| !transcript.is_empty());
        Ok(transcripts.join(" "))
    }

    fn detect_language(&self, model: &mut Model, audio: &Tensor) -> Result<u32, Error> {
        let logits = model.logits(audio, &[self.config.start])?.to_vec1::<f32>()?;

        (FIRST_LANGUAGE_TOKEN..=LAST_LANGUAGE_TOKEN)
            .max_by(|left, right| logits[*left as usize].total_cmp(&logits[*right as usize]))
            .ok_or_else(|| Error::custom("ai::audio", "Whisper has no language tokens"))
    }

    fn decode(
        &self,
        ctx: &Context,
        model: &mut Model,
        audio: &Tensor,
        language: u32,
        beams: usize,
        max_len: usize,
    ) -> Result<String, Error> {
        let transcribe = token(ctx, "<|transcribe|>")?;
        let no_timestamps = token(ctx, "<|notimestamps|>")?;
        let limit = max_len.min(self.config.max_len).max(4);
        let mut live = vec![(0f64, vec![self.config.start, language, transcribe, no_timestamps])];
        let mut done = Vec::new();

        while live.iter().any(|(_, tokens)| tokens.len() < limit) {
            let mut candidates = Vec::new();

            for (score, tokens) in &live {
                let mut probabilities = candle_nn::ops::log_softmax(&model.logits(audio, tokens)?, 0)?.to_vec1::<f32>()?;

                for token in &self.config.suppress {
                    probabilities[*token as usize] = f32::NEG_INFINITY;
                }

                if tokens.len() == 4 {
                    for token in &self.config.begin_suppress {
                        probabilities[*token as usize] = f32::NEG_INFINITY;
                    }
                }

                let mut next: Vec<_> = probabilities.iter().enumerate().collect();
                next.sort_by(|(_, left), (_, right)| right.total_cmp(left));

                for (token, probability) in next.into_iter().take(beams.max(1)) {
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

impl Transcribe for Transcriber {
    fn transcribe(
        &self,
        ctx: &Context,
        audio: &[Vec<u8>],
        language: Option<&str>,
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error> {
        audio
            .iter()
            .map(|bytes| {
                self.one(
                    ctx,
                    bytes,
                    language,
                    beams.unwrap_or(1),
                    max_len.unwrap_or(self.config.max_len),
                )
            })
            .collect()
    }
}

fn language_token(ctx: &Context, language: &str) -> Result<u32, Error> {
    if language.len() != 2 || !language.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return Err(Error::custom("ai::audio", format!("unsupported language `{language}`")));
    }

    token(ctx, &format!("<|{language}|>")).map_err(|_| Error::custom("ai::audio", format!("unsupported language `{language}`")))
}

fn token(ctx: &Context, value: &str) -> Result<u32, Error> {
    ctx.tokenizer()?
        .token_to_id(value)
        .ok_or_else(|| Error::custom("ai::audio", format!("Whisper tokenizer is missing {value}")))
}
