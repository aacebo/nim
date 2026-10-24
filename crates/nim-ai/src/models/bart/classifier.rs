use std::sync::Mutex;

use candle_core::{IndexOp, Tensor};
use candle_nn::{Embedding, Linear, Module, VarBuilder, embedding, ops};

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Forward};
use crate::pipelines::common::decode_text;
use crate::tasks::{aggregation, decode_entities};
use crate::types::{Annotation, Offset, Span};

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
    pub fn new(config: &Config, vars: VarBuilder, mode: ClassifyMode) -> Result<Self, Error> {
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

    fn text(&self, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
        text.iter()
            .map(|text| {
                let encoding = ctx.encode_one(text)?;
                let ids = encoding.get_ids();

                if ids.is_empty() {
                    return Err(Error::custom("ai::invalid", "empty classification input"));
                }

                let input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;
                let logits = Forward::forward(self, input)?;
                let last = logits.i((0, logits.dim(1)? - 1))?;
                let scores = ops::softmax(&last, 0)?.to_vec1::<f32>()?;
                let (index, score) = scores
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.total_cmp(right))
                    .ok_or_else(|| Error::custom("ai::invalid", "empty classification output"))?;

                Ok(vec![Annotation {
                    name: String::from("classification"),

                    label: self.labels[index].clone(),
                    score: *score as f64,
                    spans: vec![Span::Text(Offset::new(0, text.chars().count() as u32))],
                }])
            })
            .collect()
    }

    fn spans(&self, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
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

    fn forward(&self, input: Self::Input) -> Result<Self::Output, Error> {
        let mut encoder = self
            .encoder
            .lock()
            .map_err(|_| Error::custom("ai", "entity classification model lock poisoned"))?;

        let output = encoder.forward(&input)?;
        let output = match &self.dense {
            Some(dense) => dense.forward(&output)?.tanh()?,
            None => output,
        };

        Ok(self.classifier.forward(&output)?)
    }
}

impl Classify for Classifier {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        let text = decode_text(input)?;
        options.validate()?;

        if options.labels.is_some() {
            return Err(Error::custom(
                "ai::invalid",
                format!("{} cannot classify custom labels", ctx.name()),
            ));
        }

        if options.mode != self.mode {
            return Err(Error::custom(
                "ai::invalid",
                format!("{} cannot classify in {:?} mode", ctx.name(), options.mode),
            ));
        }

        let out = match options.mode {
            ClassifyMode::Whole => self.text(ctx, &text),
            ClassifyMode::Regions => self.spans(ctx, &text),
        }?;

        out.into_iter().map(|annotations| options.apply(annotations)).collect()
    }
}
