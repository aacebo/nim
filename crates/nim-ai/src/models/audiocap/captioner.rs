use crate::types::TextOutput;
use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::models::{Caption, CaptionOptions, Context};

use super::config::AudioCapConfig;
use super::model::AudioCapModel;

const BOS: u32 = 1;
const EOS: u32 = 2;

pub struct AudioCapCaptioner {
    model: Mutex<AudioCapModel>,
    config: AudioCapConfig,
}

impl AudioCapCaptioner {
    pub fn new(config: AudioCapConfig, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(AudioCapModel::new(&config, vars)?),
            config,
        })
    }

    fn one(&self, ctx: &Context, bytes: &[u8], beams: usize, max_len: usize) -> nim_error::Result<String> {
        let model = self
            .model
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("audio-to-text model lock poisoned"))?;
        let chunks = model.chunks(bytes, &self.config)?;
        let mut captions = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let audio = model.encode(chunk, ctx.device())?;
            captions.push(self.decode(ctx, &model, &audio, beams, max_len)?);
        }

        captions.retain(|caption| !caption.is_empty());
        Ok(captions.join(" "))
    }

    fn decode(
        &self,
        ctx: &Context,
        model: &AudioCapModel,
        audio: &candle_core::Tensor,
        beams: usize,
        max_len: usize,
    ) -> nim_error::Result<String> {
        let mut live = vec![(0f64, vec![BOS])];
        let mut done = Vec::new();

        while live.iter().any(|(_, tokens)| tokens.len() < max_len) {
            let mut candidates = Vec::new();

            for (score, tokens) in &live {
                let probabilities = candle_nn::ops::log_softmax(&model.logits(audio, tokens)?, 0)?.to_vec1::<f32>()?;
                let mut next: Vec<_> = probabilities.iter().enumerate().collect();

                next.sort_by(|(_, left), (_, right)| right.total_cmp(left));

                for (token, probability) in next.into_iter().take(beams) {
                    candidates.push((*score + *probability as f64, tokens.clone(), token as u32));
                }
            }

            candidates.sort_by(|left, right| right.0.total_cmp(&left.0));
            live.clear();

            for (score, mut tokens, token) in candidates {
                tokens.push(token);

                if token == EOS || tokens.len() >= max_len {
                    done.push((score, tokens));
                } else if live.len() < beams {
                    live.push((score, tokens));
                }
            }

            if done.len() >= beams {
                let mut scores = done.iter().map(|(score, _)| *score).collect::<Vec<_>>();

                scores.sort_by(|left, right| right.total_cmp(left));

                if live.iter().all(|(score, _)| *score <= scores[beams - 1]) {
                    break;
                }
            }
        }

        done.extend(live);
        done.sort_by(|left, right| right.0.total_cmp(&left.0));
        let tokens = done.into_iter().next().map(|(_, tokens)| tokens).unwrap_or_default();

        ctx.tokenizer()?
            .decode(&tokens, true)
            .map(|caption| caption.trim().to_string())
            .map_err(nim_error::Diagnostic::message)
    }
}

impl Caption for AudioCapCaptioner {
    fn caption<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: CaptionOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            options.validate()?;

            let beams = options.beams;
            let max_len = options.max_len;
            let beams = beams.unwrap_or(3).max(1);
            let max_len = max_len.unwrap_or(usize::MAX);
            input
                .iter()
                .map(|bytes| self.one(ctx, bytes, beams, max_len))
                .map(|result| result.map(|text| TextOutput { text, confidence: None }))
                .collect()
        })
    }
}
