use candle_core::Tensor;
use candle_nn::{Linear, Module, VarBuilder, linear, ops};

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Forward};
use crate::types::Annotation;

use super::config::Config;
use super::model::MobileNetV2;
use super::processor::Processor;

pub struct Classifier {
    model: MobileNetV2,
    classifier: Linear,
    processor: Processor,
    labels: Vec<String>,
}

impl Classifier {
    pub fn new(vars: VarBuilder, config: Config, processor: Processor) -> Result<Self, Error> {
        let labels: Vec<String> = config.id2label.clone().into_values().collect();
        let model = MobileNetV2::new(vars.clone(), &config)?;

        Ok(Self {
            model,
            classifier: linear(config.output_channels(), labels.len(), vars.pp("classifier"))?,
            processor,
            labels,
        })
    }
}

impl Forward for Classifier {
    type Input = Tensor;
    type Output = Tensor;

    fn forward(&self, input: Self::Input) -> Result<Self::Output, Error> {
        Ok(self.classifier.forward(&self.model.features(&input)?)?)
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

                let input = self.processor.tensor(input, ctx.device())?;
                let probabilities = ops::softmax(&Forward::forward(self, input)?, 1)?.to_vec2::<f32>()?;
                let mut labels: Vec<Annotation> = probabilities[0]
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
