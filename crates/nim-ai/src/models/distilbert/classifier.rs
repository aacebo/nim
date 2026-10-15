use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, Module, VarBuilder, ops};

use crate::Error;
use crate::models::{Context, EntityClassify, Forward, Label, SequenceClassify};
use crate::tasks::{aggregation, decode_entities};
use crate::types::Entity;

use super::config::Config;
use super::model::DistilBert;

pub struct Classifier {
    distilbert: DistilBert,
    pre_classifier: Option<Linear>,
    classifier: Linear,
    labels: Vec<String>,
}

impl Classifier {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self, Error> {
        let hidden = config.hidden_size();
        let labels = config.labels()?;
        let pre_classifier = vars
            .contains_tensor("pre_classifier.weight")
            .then(|| candle_nn::linear(hidden, hidden, vars.pp("pre_classifier")))
            .transpose()?;

        Ok(Self {
            distilbert: DistilBert::new(vars.clone(), config)?,
            pre_classifier,
            classifier: candle_nn::linear(hidden, labels.len(), vars.pp("classifier"))?,
            labels,
        })
    }
}

impl Forward for Classifier {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, padding): Self::Input) -> Result<Self::Output, Error> {
        let hidden = self.distilbert.forward(&ids, &padding)?;
        let hidden = match &self.pre_classifier {
            Some(layer) => layer.forward(&hidden)?.relu()?,
            None => hidden,
        };

        Ok(self.classifier.forward(&hidden)?)
    }
}

impl SequenceClassify for Classifier {
    fn classify_sequence(&self, cx: &Context, text: &[&str]) -> Result<Vec<Label>, Error> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let batch = cx.encode(text)?;
        let padding = batch.padding()?;
        let logits = Forward::forward(self, (batch.ids, padding))?.i((.., 0))?;
        let probabilities = ops::softmax(&logits, 1)?.to_vec2::<f32>()?;

        probabilities
            .into_iter()
            .map(|scores| {
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

                let shape = (1, ids.len());
                let input = Tensor::from_slice(ids, shape, cx.device())?;
                let padding = Tensor::from_slice(encoding.get_attention_mask(), shape, cx.device())?;
                let logits = Forward::forward(self, (input, padding))?;
                let probabilities = ops::softmax(&logits, 2)?.squeeze(0)?.to_vec2::<f32>()?;
                let words = aggregation::words(&probabilities, &encoding, &self.labels)?;

                Ok(decode_entities(words, text, &self.labels))
            })
            .collect()
    }
}
