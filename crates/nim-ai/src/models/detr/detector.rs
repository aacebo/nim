use std::collections::BTreeMap;

use candle_nn::{VarBuilder, ops};

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context};
use crate::types::{Annotation, BoundingBox, Span};

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

    fn regions(&self, ctx: &Context, input: &[u8], min_score: f64) -> Result<Vec<Annotation>, Error> {
        let (input, width, height) = self.processor.tensor(input, ctx.device())?;
        let (logits, boxes) = self.model.forward(&input)?;
        let probabilities = ops::softmax_last_dim(&logits)?.squeeze(0)?.to_vec2::<f32>()?;
        let boxes = boxes.squeeze(0)?.to_vec2::<f32>()?;
        let mut annotations = Vec::new();

        for (scores, bbox) in probabilities.iter().zip(boxes) {
            let Some((class_id, &score)) = scores[..scores.len() - 1]
                .iter()
                .enumerate()
                .max_by(|(left_id, left), (right_id, right)| left.total_cmp(right).then_with(|| right_id.cmp(left_id)))
            else {
                continue;
            };

            if (score as f64) < min_score {
                continue;
            }

            let Some(label) = self.labels.get(&class_id) else {
                continue;
            };
            let [center_x, center_y, w, h] =
                <[f32; 4]>::try_from(bbox).map_err(|_| Error::custom("ai::image", "invalid DETR box shape"))?;

            annotations.push(Annotation {
                name: String::from("detection"),
                label: label.clone(),
                score: score as f64,
                spans: vec![Span::Image(BoundingBox {
                    xmin: ((center_x - w / 2.0) * width as f32).clamp(0.0, width as f32),
                    ymin: ((center_y - h / 2.0) * height as f32).clamp(0.0, height as f32),
                    xmax: ((center_x + w / 2.0) * width as f32).clamp(0.0, width as f32),
                    ymax: ((center_y + h / 2.0) * height as f32).clamp(0.0, height as f32),
                })],
            });
        }

        annotations.sort_by(|left, right| right.score.total_cmp(&left.score));
        Ok(annotations)
    }
}

impl Classify for Detector {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        options.validate()?;

        if options.labels.is_some()
            || options.multi_label
            || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
        {
            return Err(Error::custom("ai::config", "DETR only supports fixed labels"));
        }

        input
            .iter()
            .map(|input| {
                if options.limit == Some(0) {
                    return Ok(Vec::new());
                }

                let annotations = self.regions(
                    ctx,
                    input,
                    if options.mode == ClassifyMode::Whole {
                        0.5
                    } else {
                        options.min_score
                    },
                )?;
                let annotations = if options.mode == ClassifyMode::Whole {
                    let mut labels = BTreeMap::<String, f64>::new();

                    for annotation in annotations {
                        labels
                            .entry(annotation.label)
                            .and_modify(|score| *score = score.max(annotation.score))
                            .or_insert(annotation.score);
                    }

                    let mut annotations: Vec<_> = labels
                        .into_iter()
                        .map(|(label, score)| Annotation {
                            name: String::from("classification"),
                            label,
                            score,
                            spans: Vec::new(),
                        })
                        .collect();
                    annotations.sort_by(|left, right| right.score.total_cmp(&left.score));
                    annotations
                } else {
                    annotations
                };

                options.apply(annotations)
            })
            .collect()
    }
}
