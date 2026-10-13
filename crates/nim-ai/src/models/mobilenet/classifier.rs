use candle_nn::ops;

use super::config::Config;
use super::model::MobileNetV2;
use super::processor::Processor;
use crate::Error;
use crate::models::{Context, Label, VisionClassify};

pub struct Classifier {
    model: MobileNetV2,
    processor: Processor,
    labels: Vec<String>,
}

impl Classifier {
    pub fn new(model: MobileNetV2, config: Config, processor: Processor) -> Self {
        Self {
            model,
            processor,
            labels: config.id2label.into_values().collect(),
        }
    }
}

impl VisionClassify for Classifier {
    fn classify_image(&self, cx: &Context, image: &[u8], limit: usize) -> Result<Vec<Label>, Error> {
        let input = self.processor.tensor(image, cx.device())?;
        let probabilities = ops::softmax(&self.model.forward(&input)?, 1)?.to_vec2::<f32>()?;
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
