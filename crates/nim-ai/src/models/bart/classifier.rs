use std::sync::Mutex;

use candle_core::{IndexOp, Tensor};
use candle_nn::{Embedding, Linear, Module, VarBuilder, embedding, ops};

use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Forward};
use crate::tasks::{aggregation, classification, decode_entities};
use crate::types::Annotation;

use super::config::Config;
use super::encoder::Encoder;

pub struct Classifier {
    encoder: Mutex<Encoder>,
    dense: Option<Linear>,
    classifier: Linear,
    labels: Vec<String>,
    mode: ClassifyMode,
}

impl Classifier {
    pub fn new(config: &Config, vars: VarBuilder, mode: ClassifyMode) -> nim_error::Result<Self> {
        let labels = config.labels()?;
        let model = vars.pp("model");
        let embed_tokens: Embedding = embedding(config.vocab_size, config.d_model, model.pp("encoder").pp("embed_tokens"))?;
        let dense = vars
            .contains_tensor("classification_head.dense.weight")
            .then(|| candle_nn::linear(config.d_model, config.d_model, vars.pp("classification_head").pp("dense")))
            .transpose()?;
        let classifier = match &dense {
            Some(_) => candle_nn::linear(config.d_model, labels.len(), vars.pp("classification_head").pp("out_proj"))?,
            None => candle_nn::linear(config.d_model, labels.len(), vars.pp("classifier"))?,
        };

        Ok(Self {
            encoder: Mutex::new(Encoder::new(config, &embed_tokens, model.pp("encoder"))?),
            dense,
            classifier,
            labels,
            mode,
        })
    }

    fn text(&self, ctx: &Context, text: &[&str]) -> nim_error::Result<Vec<Vec<Annotation>>> {
        text.iter()
            .map(|text| {
                let encoding = ctx.encode_one(text)?;
                let ids = encoding.get_ids();

                if ids.is_empty() {
                    return Err(nim_error::Diagnostic::message("empty classification input"));
                }

                let input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
                let logits = Forward::forward(self, input)?;
                let last = logits.i((0, logits.dim(1)? - 1))?;
                let scores = ops::softmax(&last, 0)?.to_vec1::<f32>()?;
                let (index, score) = scores
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.total_cmp(right))
                    .ok_or_else(|| nim_error::Diagnostic::message("empty classification output"))?;

                Ok(vec![classification(&self.labels, index, *score as f64, text)])
            })
            .collect()
    }

    fn spans(&self, ctx: &Context, text: &[&str]) -> nim_error::Result<Vec<Vec<Annotation>>> {
        text.iter()
            .map(|text| {
                let encoding = ctx.encode_one(text)?;
                let ids = encoding.get_ids();

                if ids.is_empty() {
                    return Ok(Vec::new());
                }

                let input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
                let logits = Forward::forward(self, input)?;
                let probabilities = ops::softmax(&logits, 2)?.squeeze(0)?.to_vec2::<f32>()?;
                let words = aggregation::words(&probabilities, &encoding, &self.labels)?;

                Ok(decode_entities(words, text, &self.labels))
            })
            .collect()
    }
}

impl Forward for Classifier {
    type Input = Tensor;
    type Output = Tensor;

    fn forward(&self, input: Self::Input) -> nim_error::Result<Self::Output> {
        let mut encoder = self
            .encoder
            .lock()
            .map_err(|_| nim_error::Diagnostic::message("entity classification model lock poisoned"))?;

        let output = encoder.forward(&input)?;
        let output = match &self.dense {
            Some(dense) => dense.forward(&output)?.tanh()?,
            None => output,
        };

        Ok(self.classifier.forward(&output)?)
    }
}

impl Classify for Classifier {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Annotation>>> {
        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        options.validate()?;

        if options.labels.is_some()
            || options.multi_label
            || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
        {
            return Err(nim_error::Diagnostic::message(format!(
                "{} only supports fixed labels",
                ctx.name()
            )));
        }

        if options.mode != self.mode {
            return Err(nim_error::Diagnostic::message(format!(
                "{} cannot classify in {:?} mode",
                ctx.name(),
                options.mode
            )));
        }

        let out = match options.mode {
            ClassifyMode::Whole => self.text(ctx, &text),
            ClassifyMode::Regions => self.spans(ctx, &text),
        }?;

        out.into_iter().map(|annotations| options.apply(annotations)).collect()
    }
}
