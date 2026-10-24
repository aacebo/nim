use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{Context, ImageToText};

use super::config::Config;
use super::model::Model;
use super::processor::Processor;

pub struct Captioner {
    model: Mutex<Model>,
    processor: Processor,
    config: Generation,
}

#[derive(Clone, Copy)]
struct Generation {
    bos: u32,
    eos: u32,
    max_len: usize,
}

impl Captioner {
    pub fn new(config: Config, processor: Processor, vars: VarBuilder) -> Result<Self, Error> {
        let generation = Generation {
            bos: config.bos_token_id,
            eos: config.eos_token_id,
            max_len: config.max_position_embeddings,
        };

        Ok(Self {
            model: Mutex::new(Model::new(&config, vars)?),
            processor,
            config: generation,
        })
    }

    fn one(&self, ctx: &Context, bytes: &[u8], beams: usize, max_len: usize) -> Result<String, Error> {
        let image = self.processor.tensor(bytes, ctx.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "image-to-text model lock poisoned"))?;
        let visual = model.vision(&image)?;
        let limit = max_len.min(self.config.max_len).max(1);
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

            for (score, mut tokens, token) in candidates.into_iter().take(beams.max(1) * 2) {
                tokens.push(token);

                if token == self.config.eos {
                    done.push((score, tokens));
                } else if live.len() < beams.max(1) {
                    live.push((score, tokens));
                }

                if live.len() >= beams.max(1) && done.len() >= beams.max(1) {
                    break;
                }
            }

            if live.is_empty() {
                break;
            }
        }

        done.extend(live);
        done.sort_by(|a, b| b.0.total_cmp(&a.0));
        let tokens = done.into_iter().next().map(|(_, tokens)| tokens).unwrap_or_default();

        ctx.tokenizer()?
            .decode(&tokens, true)
            .map(|text| text.trim().to_string())
            .map_err(|e| Error::custom("ai::decoding", e))
    }
}

impl ImageToText for Captioner {
    fn image_to_text(
        &self,
        ctx: &Context,
        images: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error> {
        images
            .iter()
            .map(|image| self.one(ctx, image, beams.unwrap_or(1), max_len.unwrap_or(20)))
            .collect()
    }
}
