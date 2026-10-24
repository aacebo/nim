use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context};
use crate::types::Annotation;

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

impl Classify for Classifier {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        options.validate()?;

        if options.mode != ClassifyMode::Whole
            || options.labels.is_some()
            || options.multi_label
            || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
        {
            return Err(Error::custom(
                "ai::config",
                "this classifier only supports whole-input classification with fixed labels",
            ));
        }

        input
            .iter()
            .map(|input| {
                if options.limit == Some(0) {
                    return Ok(Vec::new());
                }

                let model = self
                    .model
                    .lock()
                    .map_err(|_| Error::custom("ai", "audio classification model lock poisoned"))?;
                let probabilities = model.forward(input, &self.config, ctx.device())?.to_vec2::<f32>()?;
                let mut labels: Vec<_> = probabilities[0]
                    .iter()
                    .enumerate()
                    .map(|(index, score)| Annotation {
                        name: String::from("classification"),
                        spans: Vec::new(),
                        label: self.labels[index].clone(),
                        score: *score as f64,
                    })
                    .collect();

                labels.sort_by(|left, right| right.score.total_cmp(&left.score));
                options.apply(labels)
            })
            .collect()
    }
}
