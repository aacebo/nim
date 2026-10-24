use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, Module, VarBuilder, ops};

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Forward};
use crate::pipelines::common::decode_text;
use crate::tasks::{aggregation, decode_entities};
use crate::types::{Annotation, Offset, Span};

use super::config::Config;
use super::model::Bert;

pub struct Classifier {
    bert: Bert,
    pooler: Option<Linear>,
    classifier: Linear,
    labels: Vec<String>,
    mode: ClassifyMode,
}

impl Classifier {
    pub fn new(vars: VarBuilder, config: &Config, mode: ClassifyMode) -> Result<Self, Error> {
        let labels = config.labels()?;
        let pooler = vars
            .contains_tensor("pooler.dense.weight")
            .then(|| candle_nn::linear(config.hidden_size, config.hidden_size, vars.pp("pooler").pp("dense")))
            .transpose()?;
        let classifier = candle_nn::linear(config.hidden_size, labels.len(), vars.pp("classifier"))?;

        Ok(Self {
            bert: Bert::new(vars, config)?,
            pooler,
            classifier,
            labels,
            mode,
        })
    }

    fn text(&self, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let batch = ctx.encode(text)?;
        let logits = Forward::forward(self, (batch.ids, batch.mask))?.i((.., 0))?;
        let probabilities = ops::softmax(&logits, 1)?.to_vec2::<f32>()?;

        probabilities
            .into_iter()
            .zip(text)
            .map(|(scores, text)| {
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

                let shape = (1, ids.len());
                let input = Tensor::from_slice(ids, shape, ctx.device())?;
                let mask = Tensor::from_slice(encoding.get_attention_mask(), shape, ctx.device())?;
                let logits = Forward::forward(self, (input, mask))?;
                let probabilities = ops::softmax(&logits, 2)?.squeeze(0)?.to_vec2::<f32>()?;
                let words = aggregation::words(&probabilities, &encoding, &self.labels)?;

                Ok(decode_entities(words, text, &self.labels))
            })
            .collect()
    }
}

impl Forward for Classifier {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, mask): Self::Input) -> Result<Self::Output, Error> {
        let hidden = self.bert.forward(&ids, &mask)?;
        let hidden = match &self.pooler {
            Some(pooler) => pooler.forward(&hidden)?.tanh()?,
            None => hidden,
        };

        Ok(self.classifier.forward(&hidden)?)
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
