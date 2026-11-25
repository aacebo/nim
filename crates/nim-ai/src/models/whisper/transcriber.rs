use crate::types::TextOutput;
use std::sync::Mutex;

use candle_core::Tensor;
use candle_nn::VarBuilder;

use crate::models::{Context, Transcribe, TranscribeOptions};

use super::config::WhisperConfig;
use super::model::WhisperModel;
use super::processor::WhisperProcessor;

const FIRST_LANGUAGE_TOKEN: u32 = 50_259;
const LAST_LANGUAGE_TOKEN: u32 = 50_357;

pub struct WhisperTranscriber {
    model: Mutex<WhisperModel>,
    processor: WhisperProcessor,
    model_config: WhisperConfig,
    config: WhisperGenerationConfig,
}

#[derive(Clone)]
struct WhisperGenerationConfig {
    start: u32,
    eos: u32,
    max_positions: usize,
    suppress: Vec<u32>,
    begin_suppress: Vec<u32>,
}

impl WhisperTranscriber {
    pub fn new(config: WhisperConfig, processor: WhisperProcessor, vars: VarBuilder) -> nim_error::Result<Self> {
        let decoding = WhisperGenerationConfig {
            start: config.decoder_start_token_id,
            eos: config.eos_token_id,
            max_positions: config.model().max_target_positions,
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

    fn one(
        &self,
        ctx: &Context,
        bytes: &[u8],
        language: Option<&str>,
        beams: usize,
        max_tokens: usize,
    ) -> nim_error::Result<String> {
        let chunks = self.processor.tensors(bytes, &self.model_config, ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("transcription model lock poisoned"))?;
        let first = model.encode(&chunks[0])?;
        let language = match language {
            Some(language) => language_token(ctx, language)?,
            None => self.detect_language(&mut model, &first)?,
        };
        let (transcript, used) = self.decode(ctx, &mut model, &first, language, beams, max_tokens)?;
        let mut remaining = max_tokens - used;
        let mut transcripts = vec![transcript];

        for chunk in chunks.iter().skip(1) {
            if remaining == 0 {
                break;
            }

            let audio = model.encode(chunk)?;
            let (transcript, used) = self.decode(ctx, &mut model, &audio, language, beams, remaining)?;
            remaining -= used;
            transcripts.push(transcript);
        }

        transcripts.retain(|transcript| !transcript.is_empty());
        Ok(transcripts.join(" "))
    }

    fn detect_language(&self, model: &mut WhisperModel, audio: &Tensor) -> nim_error::Result<u32> {
        let logits = model.logits(audio, &[self.config.start])?.to_vec1::<f32>()?;

        (FIRST_LANGUAGE_TOKEN..=LAST_LANGUAGE_TOKEN)
            .max_by(|left, right| logits[*left as usize].total_cmp(&logits[*right as usize]))
            .ok_or_else(|| nim_error::Diagnostic::message("Whisper has no language tokens"))
    }

    fn decode(
        &self,
        ctx: &Context,
        model: &mut WhisperModel,
        audio: &Tensor,
        language: u32,
        beams: usize,
        max_tokens: usize,
    ) -> nim_error::Result<(String, usize)> {
        let transcribe = token(ctx, "<|transcribe|>")?;
        let no_timestamps = token(ctx, "<|notimestamps|>")?;
        let limit = 4 + crate::models::output_limit("Whisper", self.config.max_positions, 4, Some(max_tokens))?;
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
        let used = tokens.len().saturating_sub(4);
        let text = ctx
            .tokenizer()?
            .decode(&tokens, true)
            .map_err(nim_error::Diagnostic::message)?;

        Ok((text.trim().to_string(), used))
    }
}

impl Transcribe for WhisperTranscriber {
    fn transcribe<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: TranscribeOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            options.validate()?;

            let beams = options.beams;
            let max_tokens = options.max_tokens;
            let language = options.language;
            input
                .iter()
                .map(|bytes| {
                    self.one(
                        ctx,
                        bytes,
                        language,
                        beams.unwrap_or(1),
                        max_tokens.unwrap_or(crate::models::DEFAULT_MAX_TOKENS),
                    )
                })
                .map(|result| result.map(|text| TextOutput { text, confidence: None }))
                .collect()
        })
    }
}

fn language_token(ctx: &Context, language: &str) -> nim_error::Result<u32> {
    if language.len() != 2 || !language.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return Err(nim_error::Diagnostic::message(format!("unsupported language `{language}`")));
    }

    token(ctx, &format!("<|{language}|>"))
        .map_err(|_| nim_error::Diagnostic::message(format!("unsupported language `{language}`")))
}

fn token(ctx: &Context, value: &str) -> nim_error::Result<u32> {
    ctx.tokenizer()?
        .token_to_id(value)
        .ok_or_else(|| nim_error::Diagnostic::message(format!("Whisper tokenizer is missing {value}")))
}
