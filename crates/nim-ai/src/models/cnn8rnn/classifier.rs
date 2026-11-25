use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::types::{Classification, Span, TimeRange};
use crate::{Classify, ClassifyMode, ClassifyOptions, Context};

use super::config::Cnn8RnnConfig;
use super::model::Cnn8RnnModel;

pub struct Cnn8RnnClassifier {
    model: Mutex<Cnn8RnnModel>,
    config: Cnn8RnnConfig,
    labels: Vec<String>,
}

impl Cnn8RnnClassifier {
    pub fn new(config: Cnn8RnnConfig, labels: Vec<String>, vars: VarBuilder) -> nim_error::Result<Self> {
        Ok(Self {
            model: Mutex::new(Cnn8RnnModel::new(&config, vars)?),
            config,
            labels,
        })
    }
}

impl Classify for Cnn8RnnClassifier {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        Box::pin(async move {
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
                        .map(|(index, score)| Classification {
                            name: String::from("classification"),
                            spans: vec![Span::Time(TimeRange {
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
        })
    }
}
