use candle_core::Tensor;
use candle_nn::{Linear, Module, VarBuilder, linear, ops};

use crate::Error;
use crate::models::{Context, Forward, ImageClassify, Label};

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

impl ImageClassify for Classifier {
    fn classify_image(&self, ctx: &Context, image: &[u8], limit: usize) -> Result<Vec<Label>, Error> {
        let input = self.processor.tensor(image, ctx.device())?;
        let probabilities = ops::softmax(&Forward::forward(self, input)?, 1)?.to_vec2::<f32>()?;
        let mut labels: Vec<Label> = probabilities[0]
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
