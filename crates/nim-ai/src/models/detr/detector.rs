use std::collections::BTreeMap;

use candle_nn::{VarBuilder, ops};

use crate::Error;
use crate::models::{BoundingBox, Context, Detection, ImageClassify, ImageDetect, Label};

use super::{Config, Model, Processor};

pub struct Detector {
    model: Model,
    processor: Processor,
    labels: BTreeMap<usize, String>,
}

impl Detector {
    pub fn new(vars: VarBuilder, config: Config, processor: Processor) -> Result<Self, Error> {
        Ok(Self {
            model: Model::new(&config, vars)?,
            processor,
            labels: config.id2label,
        })
    }
}

impl ImageDetect for Detector {
    fn detect_image(&self, cx: &Context, image: &[u8], min_score: f64, limit: usize) -> Result<Vec<Detection>, Error> {
        if !min_score.is_finite() || !(0.0..=1.0).contains(&min_score) {
            return Err(Error::custom("ai::config", "min_score must be finite and between 0 and 1"));
        }

        if limit == 0 {
            return Ok(Vec::new());
        }

        let (input, width, height) = self.processor.tensor(image, cx.device())?;
        let (logits, boxes) = self.model.forward(&input)?;
        let probabilities = ops::softmax_last_dim(&logits)?.squeeze(0)?.to_vec2::<f32>()?;
        let boxes = boxes.squeeze(0)?.to_vec2::<f32>()?;
        let mut detections = Vec::new();

        for (scores, bbox) in probabilities.iter().zip(boxes) {
            let Some((class_id, &score)) = scores[..scores.len() - 1]
                .iter()
                .enumerate()
                .max_by(|(left_id, left), (right_id, right)| left.total_cmp(right).then_with(|| right_id.cmp(left_id)))
            else {
                continue;
            };

            if score as f64 <= min_score {
                continue;
            }

            let Some(label) = self.labels.get(&class_id) else {
                continue;
            };

            let [cx, cy, w, h] = <[f32; 4]>::try_from(bbox).map_err(|_| Error::custom("ai::image", "invalid DETR box shape"))?;

            detections.push(Detection {
                class_id,
                label: label.clone(),
                score: score as f64,
                bbox: BoundingBox {
                    xmin: ((cx - w / 2.0) * width as f32).clamp(0.0, width as f32),
                    ymin: ((cy - h / 2.0) * height as f32).clamp(0.0, height as f32),
                    xmax: ((cx + w / 2.0) * width as f32).clamp(0.0, width as f32),
                    ymax: ((cy + h / 2.0) * height as f32).clamp(0.0, height as f32),
                },
            });
        }

        detections.sort_by(|left, right| right.score.total_cmp(&left.score));
        detections.truncate(limit);
        Ok(detections)
    }
}

impl ImageClassify for Detector {
    fn classify_image(&self, cx: &Context, image: &[u8], limit: usize) -> Result<Vec<Label>, Error> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let detections = self.detect_image(cx, image, 0.5, usize::MAX)?;
        let mut labels = BTreeMap::<String, f64>::new();

        for detection in detections {
            labels
                .entry(detection.label)
                .and_modify(|score| *score = score.max(detection.score))
                .or_insert(detection.score);
        }

        let mut labels: Vec<_> = labels.into_iter().map(|(label, score)| Label { label, score }).collect();
        labels.sort_by(|left, right| right.score.total_cmp(&left.score));
        labels.truncate(limit);

        Ok(labels)
    }
}
