use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{AudioClassify, Context, Label};

use super::config::Config;
use super::model::Model;

pub struct Classifier {
    model: Mutex<Model>,
    config: Config,
    labels: Vec<String>,
}

impl Classifier {
    pub fn new(config: Config, labels: Vec<String>, vars: VarBuilder) -> Result<Self, Error> {
        Ok(Self {
            model: Mutex::new(Model::new(&config, vars)?),
            config,
            labels,
        })
    }
}

impl AudioClassify for Classifier {
    fn classify_audio(&self, ctx: &Context, audio: &[u8], limit: usize) -> Result<Vec<Label>, Error> {
        let model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "audio classification model lock poisoned"))?;
        let probabilities = model.forward(audio, &self.config, ctx.device())?.to_vec2::<f32>()?;
        let mut labels: Vec<_> = probabilities[0]
            .iter()
            .enumerate()
            .map(|(index, score)| Label {
                label: self.labels[index].clone(),
                score: *score as f64,
            })
            .collect();

        labels.sort_by(|left, right| right.score.total_cmp(&left.score));
        labels.truncate(limit.min(labels.len()));
        Ok(labels)
    }
}
