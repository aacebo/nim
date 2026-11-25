use std::collections::BTreeMap;

use candle_nn::{VarBuilder, ops};

use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context};
use crate::types::{BoundingBox, Classification, Span};

use super::{DetrConfig, DetrModel, DetrProcessor};

pub struct DetrDetector {
    model: DetrModel,
    processor: DetrProcessor,
    labels: BTreeMap<usize, String>,
}

impl DetrDetector {
    pub fn new(vars: VarBuilder, config: DetrConfig, processor: DetrProcessor) -> nim_error::Result<Self> {
        Ok(Self {
            model: DetrModel::new(&config, vars)?,
            processor,
            labels: config.id2label,
        })
    }

    fn regions(&self, ctx: &Context, input: &[u8], min_score: f64) -> nim_error::Result<Vec<Classification>> {
        let (input, width, height) = self.processor.tensor(input, ctx.device())?;
        let (logits, boxes) = self.model.forward(&input)?;
        let probabilities = ops::softmax_last_dim(&logits)?.squeeze(0)?.to_vec2::<f32>()?;
        let boxes = boxes.squeeze(0)?.to_vec2::<f32>()?;
        let mut classifications = Vec::new();

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
                <[f32; 4]>::try_from(bbox).map_err(|_| nim_error::Diagnostic::message("invalid DETR box shape"))?;

            classifications.push(Classification {
                name: String::from("detection"),
                label: label.clone(),
                score: score as f64,
                spans: vec![Span::Space(BoundingBox {
                    xmin: ((center_x - w / 2.0) * width as f32).clamp(0.0, width as f32),
                    ymin: ((center_y - h / 2.0) * height as f32).clamp(0.0, height as f32),
                    xmax: ((center_x + w / 2.0) * width as f32).clamp(0.0, width as f32),
                    ymax: ((center_y + h / 2.0) * height as f32).clamp(0.0, height as f32),
                })],
            });
        }

        classifications.sort_by(|left, right| right.score.total_cmp(&left.score));
        Ok(classifications)
    }
}

impl Classify for DetrDetector {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        Box::pin(async move {
            options.validate()?;

            if options.labels.is_some()
                || options.multi_label
                || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
            {
                return Err(nim_error::Diagnostic::message("DETR only supports fixed labels"));
            }

            input
                .iter()
                .map(|input| {
                    if options.limit == Some(0) {
                        return Ok(Vec::new());
                    }

                    let classifications = self.regions(
                        ctx,
                        input,
                        if options.mode == ClassifyMode::Whole {
                            0.5
                        } else {
                            options.min_score
                        },
                    )?;
                    let classifications = if options.mode == ClassifyMode::Whole {
                        let mut labels = BTreeMap::<String, f64>::new();

                        for classification in classifications {
                            labels
                                .entry(classification.label)
                                .and_modify(|score| *score = score.max(classification.score))
                                .or_insert(classification.score);
                        }

                        let mut classifications: Vec<_> = labels
                            .into_iter()
                            .map(|(label, score)| Classification {
                                name: String::from("classification"),
                                label,
                                score,
                                spans: Vec::new(),
                            })
                            .collect();
                        classifications.sort_by(|left, right| right.score.total_cmp(&left.score));
                        classifications
                    } else {
                        classifications
                    };

                    options.apply(classifications)
                })
                .collect()
        })
    }
}
