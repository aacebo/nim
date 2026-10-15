use std::sync::Mutex;

use candle_core::{IndexOp, Tensor};
use candle_nn::{Embedding, Linear, Module, VarBuilder, embedding, ops};

use crate::Error;
use crate::models::{Context, EntityClassify, Forward, Label, SequenceClassify};
use crate::tasks::{aggregation, decode_entities};
use crate::types::Entity;

use super::config::Config;
use super::encoder::Encoder;

pub struct Classifier {
    encoder: Mutex<Encoder>,
    dense: Option<Linear>,
    classifier: Linear,
    labels: Vec<String>,
}

impl Classifier {
    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
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
        })
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

impl SequenceClassify for Classifier {
    fn classify_sequence(&self, cx: &Context, text: &[&str]) -> Result<Vec<Label>, Error> {
        text.iter()
            .map(|text| {
                let encoding = cx.encode_one(text)?;
                let ids = encoding.get_ids();

                if ids.is_empty() {
                    return Err(Error::custom("ai::invalid", "empty classification input"));
                }

                let input = Tensor::from_slice(ids, (1, ids.len()), cx.device())?;
                let logits = Forward::forward(self, input)?;
                let last = logits.i((0, logits.dim(1)? - 1))?;
                let scores = ops::softmax(&last, 0)?.to_vec1::<f32>()?;
                let (index, score) = scores
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.total_cmp(right))
                    .ok_or_else(|| Error::custom("ai::invalid", "empty classification output"))?;

                Ok(Label {
                    label: self.labels[index].clone(),
                    score: *score as f64,
                })
            })
            .collect()
    }
}

impl EntityClassify for Classifier {
    fn classify_entities(&self, cx: &Context, text: &[&str]) -> Result<Vec<Vec<Entity>>, Error> {
        text.iter()
            .map(|text| {
                let encoding = cx.encode_one(text)?;
                let ids = encoding.get_ids();

                if ids.is_empty() {
                    return Ok(Vec::new());
                }

                let input = Tensor::from_slice(ids, (1, ids.len()), cx.device())?;
                let logits = Forward::forward(self, input)?;
                let probabilities = ops::softmax(&logits, 2)?.squeeze(0)?.to_vec2::<f32>()?;
                let words = aggregation::words(&probabilities, &encoding, &self.labels)?;

                Ok(decode_entities(words, text, &self.labels))
            })
            .collect()
    }
}
