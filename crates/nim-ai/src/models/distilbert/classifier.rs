use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, Module, VarBuilder, ops};

use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Forward};
use crate::tasks::{aggregation, classification, decode_entities};
use crate::types::Classification;

use super::config::Config;
use super::model::DistilBert;

pub struct Classifier {
    distilbert: DistilBert,
    pre_classifier: Option<Linear>,
    classifier: Linear,
    labels: Vec<String>,
    mode: ClassifyMode,
}

impl Classifier {
    pub fn new(vars: VarBuilder, config: &Config, mode: ClassifyMode) -> nim_error::Result<Self> {
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
            mode,
        })
    }

    fn text(&self, ctx: &Context, text: &[&str]) -> nim_error::Result<Vec<Vec<Classification>>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let batch = ctx.encode(text)?;
        let padding = batch.padding()?;
        let logits = Forward::forward(self, (batch.ids, padding))?.i((.., 0))?;
        let probabilities = ops::softmax(&logits, 1)?.to_vec2::<f32>()?;

        probabilities
            .into_iter()
            .zip(text)
            .map(|(scores, text)| {
                let (index, score) = scores
                    .iter()
                    .enumerate()
                    .max_by(|(_, left), (_, right)| left.total_cmp(right))
                    .ok_or_else(|| nim_error::Diagnostic::message("empty classification output"))?;

                Ok(vec![classification(&self.labels, index, *score as f64, text)])
            })
            .collect()
    }

    fn spans(&self, ctx: &Context, text: &[&str]) -> nim_error::Result<Vec<Vec<Classification>>> {
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
                let padding = (mask.ones_like()? - mask)?;
                let logits = Forward::forward(self, (input, padding))?;
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

    fn forward(&self, (ids, padding): Self::Input) -> nim_error::Result<Self::Output> {
        let hidden = self.distilbert.forward(&ids, &padding)?;
        let hidden = match &self.pre_classifier {
            Some(layer) => layer.forward(&hidden)?.relu()?,
            None => hidden,
        };

        Ok(self.classifier.forward(&hidden)?)
    }
}

impl Classify for Classifier {
    fn classify(
        &self,
        ctx: &Context,
        input: &[&[u8]],
        options: ClassifyOptions<'_>,
    ) -> nim_error::Result<Vec<Vec<Classification>>> {
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

        out.into_iter()
            .map(|classifications| options.apply(classifications))
            .collect()
    }
}
