use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::types::TextOutput;
use crate::{Caption, CaptionOptions, Context};

use super::config::GitConfig;
use super::model::GitModel;
use super::processor::GitProcessor;

pub struct GitCaptioner {
    model: Mutex<GitModel>,
    processor: GitProcessor,
    config: GitGenerationConfig,
}

#[derive(Clone, Copy)]
struct GitGenerationConfig {
    bos: u32,
    eos: u32,
    max_positions: usize,
}

impl GitCaptioner {
    pub fn new(config: GitConfig, processor: GitProcessor, vars: VarBuilder) -> nim_error::Result<Self> {
        let decoding = GitGenerationConfig {
            bos: config.bos_token_id,
            eos: config.eos_token_id,
            max_positions: config.max_position_embeddings,
        };

        Ok(Self {
            model: Mutex::new(GitModel::new(&config, vars)?),
            processor,
            config: decoding,
        })
    }

    fn one(&self, ctx: &Context, bytes: &[u8], beams: usize, max_tokens: usize) -> nim_error::Result<String> {
        let image = self.processor.tensor(bytes, ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("image-to-text model lock poisoned"))?;
        let visual = model.vision(&image)?;
        let limit = crate::models::output_limit("GIT", self.config.max_positions, 1, Some(max_tokens))?;
        let beams = beams.max(1);
        let mut live = vec![(0f64, vec![self.config.bos])];
        let mut done = Vec::new();

        for _ in 0..limit {
            let mut candidates = Vec::new();

            for (score, tokens) in &live {
                let logits = model.logits(&visual, tokens)?;
                let probabilities = candle_nn::ops::log_softmax(&logits, 1)?.to_vec2::<f32>()?;

                for (token, probability) in probabilities[0].iter().enumerate() {
                    candidates.push((*score + *probability as f64, tokens.clone(), token as u32));
                }
            }

            candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
            live.clear();

            for (score, mut tokens, token) in candidates.into_iter().take(if beams == 1 { 1 } else { beams * 2 }) {
                tokens.push(token);

                if token == self.config.eos {
                    done.push((score, tokens));
                } else if live.len() < beams {
                    live.push((score, tokens));
                }

                if live.len() >= beams && done.len() >= beams {
                    break;
                }
            }

            if live.is_empty() || done.iter().any(|(score, _)| live.iter().all(|(other, _)| score >= other)) {
                break;
            }
        }

        done.extend(live);
        done.sort_by(|a, b| b.0.total_cmp(&a.0));
        let tokens = done.into_iter().next().map(|(_, tokens)| tokens).unwrap_or_default();

        ctx.tokenizer()?
            .decode(&tokens, true)
            .map(|text| text.trim().to_string())
            .map_err(nim_error::Diagnostic::message)
    }
}

impl Caption for GitCaptioner {
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
                .map(|image| {
                    self.one(
                        ctx,
                        image,
                        beams.unwrap_or(1),
                        max_tokens.unwrap_or(crate::models::DEFAULT_MAX_TOKENS),
                    )
                })
                .map(|result| result.map(|text| TextOutput { text, confidence: None }))
                .collect()
        })
    }
}
