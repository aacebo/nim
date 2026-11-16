use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context};
use crate::types::{Annotation, Span, TimeSpan};

use super::config::Config;
use super::model::Model;

pub struct Classifier {
    model: Mutex<Model>,
    config: Config,
    labels: Vec<String>,
}

impl Classifier {
    pub fn new(config: Config, labels: Vec<String>, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Model::new(&config, vars)?),
            config,
            labels,
        })
    }
}

impl Classify for Classifier {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Annotation>>> {
        options.validate()?;

        if options.mode != ClassifyMode::Whole
            || options.labels.is_some()
            || options.multi_label
            || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
        {
            return Err(nim_error::Diagnostic::message(
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
                    .map_err(|_| nim_error::Diagnostic::message("audio classification model lock poisoned"))?;
                let (probabilities, duration) = model.forward(input, &self.config, ctx.device())?;
                let probabilities = probabilities.to_vec2::<f32>()?;
                let mut labels: Vec<_> = probabilities[0]
                    .iter()
                    .enumerate()
                    .map(|(index, score)| Annotation {
                        name: String::from("classification"),
                        spans: vec![Span::Audio(TimeSpan {
                            begin: 0.0,
                            end: duration,
                        })],
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
