use candle_core::Tensor;
use candle_nn::{Linear, Module, VarBuilder, linear, ops};

use crate::types::{Classification, Span};
use crate::{Classify, ClassifyMode, ClassifyOptions, Context, models::Forward};

use super::config::MobileNetConfig;
use super::model::MobileNetV2;
use super::processor::MobileNetProcessor;

pub struct MobileNetClassifier {
    model: MobileNetV2,
    classifier: Linear,
    processor: MobileNetProcessor,
    labels: Vec<String>,
}

impl MobileNetClassifier {
    pub fn new(vars: VarBuilder, config: MobileNetConfig, processor: MobileNetProcessor) -> nim_error::Result<Self> {
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

impl Forward for MobileNetClassifier {
    type Input = Tensor;
    type Output = Tensor;

    fn forward(&self, input: Self::Input) -> nim_error::Result<Self::Output> {
        Ok(self.classifier.forward(&self.model.features(&input)?)?)
    }
}

impl Classify for MobileNetClassifier {
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

                    let (input, bounds) = self.processor.tensor(input, ctx.device())?;
                    let probabilities = ops::softmax(&Forward::forward(self, input)?, 1)?.to_vec2::<f32>()?;
                    let mut labels: Vec<Classification> = probabilities[0]
                        .iter()
                        .enumerate()
                        .map(|(index, score)| Classification {
                            name: String::from("classification"),
                            spans: vec![Span::Space(bounds)],
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
