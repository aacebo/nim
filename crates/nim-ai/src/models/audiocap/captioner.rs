use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{AudioToText, Context};

use super::config::Config;
use super::model::Model;

const BOS: u32 = 1;
const EOS: u32 = 2;
const MAX_LEN: usize = 20;

pub struct Captioner {
    model: Mutex<Model>,
    config: Config,
}

impl Captioner {
    pub fn new(config: Config, vars: VarBuilder) -> Result<Self, Error> {
        Ok(Self {
            model: Mutex::new(Model::new(&config, vars)?),
            config,
        })
    }

    fn one(&self, cx: &Context, bytes: &[u8], beams: usize, max_len: usize) -> Result<String, Error> {
        let model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "audio-to-text model lock poisoned"))?;
        let chunks = model.chunks(bytes, &self.config)?;
        let mut captions = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let audio = model.encode(chunk, cx.device())?;
            captions.push(self.decode(cx, &model, &audio, beams, max_len)?);
        }

        captions.retain(|caption| !caption.is_empty());
        Ok(captions.join(" "))
    }

    fn decode(
        &self,
        cx: &Context,
        model: &Model,
        audio: &candle_core::Tensor,
        beams: usize,
        max_len: usize,
    ) -> Result<String, Error> {
        let mut live = vec![(0f64, vec![BOS])];
        let mut done = Vec::new();

        while live.iter().any(|(_, tokens)| tokens.len() < max_len) {
            let mut candidates = Vec::new();

            for (score, tokens) in &live {
                let probabilities = candle_nn::ops::log_softmax(&model.logits(&audio, tokens)?, 0)?.to_vec1::<f32>()?;
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
        }

        done.extend(live);
        done.sort_by(|left, right| right.0.total_cmp(&left.0));
        let tokens = done.into_iter().next().map(|(_, tokens)| tokens).unwrap_or_default();

        cx.tokenizer()?
            .decode(&tokens, true)
            .map(|caption| caption.trim().to_string())
            .map_err(|err| Error::custom("ai::decoding", err))
    }
}

impl AudioToText for Captioner {
    fn audio_to_text(
        &self,
        cx: &Context,
        audio: &[Vec<u8>],
        beams: Option<usize>,
        max_len: Option<usize>,
    ) -> Result<Vec<String>, Error> {
        let beams = beams.unwrap_or(3).max(1);
        let max_len = max_len.unwrap_or(MAX_LEN).min(MAX_LEN).max(1);

        audio.iter().map(|bytes| self.one(cx, bytes, beams, max_len)).collect()
    }
}
