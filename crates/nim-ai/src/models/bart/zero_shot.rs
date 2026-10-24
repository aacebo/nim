use std::sync::Mutex;

use candle_core::{IndexOp, Tensor};
use candle_nn::{Linear, VarBuilder, embedding, linear};
use tokenizers::{TruncationParams, TruncationStrategy};

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Forward};
use crate::tasks::zero_shot::{self, softmax};
use crate::types::{Annotation, Offset};

use super::config::Config;
use super::decoder::Decoder;
use super::encoder::Encoder;

pub struct ZeroShotClassifier {
    model: Mutex<(Encoder, Decoder)>,
    dense: Linear,
    classifier: Linear,
    labels: Vec<String>,
    entailment: usize,
    contradiction: usize,
    eos_token_id: u32,
    decoder_start_token_id: u32,
    max_positions: usize,
}

impl ZeroShotClassifier {
    pub fn supports(config: &Config) -> bool {
        ["entailment", "contradiction"]
            .iter()
            .all(|name| config.id2label.values().any(|label| label.eq_ignore_ascii_case(name)))
    }

    pub fn new(config: &Config, vars: VarBuilder) -> Result<Self, Error> {
        let labels = config.labels()?;
        let index = |name: &str| {
            labels
                .iter()
                .position(|label| label.eq_ignore_ascii_case(name))
                .ok_or_else(|| Error::custom("ai::invalid", format!("NLI model is missing {name}")))
        };
        let entailment = index("entailment")?;
        let contradiction = index("contradiction")?;
        let model = vars.pp("model");
        let shared = model.pp("shared");
        let tokens = embedding(
            config.vocab_size,
            config.d_model,
            if shared.contains_tensor("weight") {
                shared
            } else {
                model.pp("encoder").pp("embed_tokens")
            },
        )?;

        Ok(Self {
            model: Mutex::new((
                Encoder::new(config, &tokens, model.pp("encoder"))?,
                Decoder::new(config, &tokens, model.pp("decoder"))?,
            )),
            dense: linear(config.d_model, config.d_model, vars.pp("classification_head").pp("dense"))?,
            classifier: linear(config.d_model, labels.len(), vars.pp("classification_head").pp("out_proj"))?,
            labels,
            entailment,
            contradiction,
            eos_token_id: config.eos_token_id,
            decoder_start_token_id: config.decoder_start_token_id,
            max_positions: config.max_position_embeddings,
        })
    }

    fn logits(&self, ctx: &Context, ids: &[u32]) -> Result<Vec<f32>, Error> {
        let input = Tensor::from_slice(ids, (1, ids.len()), ctx.device())?;

        Ok(Forward::forward(self, input)?.squeeze(0)?.to_vec1::<f32>()?)
    }

    fn custom_text(&self, ctx: &Context, text: &[&str], options: &ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        let mut tokenizer = ctx.tokenizer()?.clone();
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: self.max_positions,
                strategy: TruncationStrategy::OnlyFirst,
                ..Default::default()
            }))
            .map_err(|err| Error::custom("ai::encoding", err))?;

        let labels = options
            .labels
            .ok_or_else(|| Error::custom("ai::config", "custom labels are required"))?;

        text.iter()
            .map(|text| {
                let logits = labels
                    .iter()
                    .map(|label| {
                        let hypothesis = options.hypothesis_template.replace("{}", label);
                        let encoding = tokenizer
                            .encode((*text, hypothesis.as_str()), true)
                            .map_err(|err| Error::custom("ai::encoding", err))?;
                        self.logits(ctx, encoding.get_ids())
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                let scores = if options.multi_label || labels.len() == 1 {
                    logits
                        .iter()
                        .map(|row| softmax(&[row[self.contradiction] as f64, row[self.entailment] as f64])[1])
                        .collect()
                } else {
                    softmax(&logits.iter().map(|row| row[self.entailment] as f64).collect::<Vec<_>>())
                };

                Ok(labels
                    .iter()
                    .zip(scores)
                    .map(|(label, score)| Annotation {
                        name: String::from("classification"),
                        text: text.to_string(),
                        label: label.clone(),
                        score,
                        spans: vec![Offset::new(0, text.chars().count() as u32)],
                    })
                    .collect())
            })
            .collect()
    }

    fn text(&self, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
        let mut tokenizer = ctx.tokenizer()?.clone();
        tokenizer.with_padding(None);
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: self.max_positions,
                ..Default::default()
            }))
            .map_err(|err| Error::custom("ai::encoding", err))?;

        text.iter()
            .map(|text| {
                let encoding = tokenizer
                    .encode(*text, true)
                    .map_err(|err| Error::custom("ai::encoding", err))?;
                let logits = self.logits(ctx, encoding.get_ids())?;
                let scores = softmax(&logits.into_iter().map(f64::from).collect::<Vec<_>>());
                let (index, score) = scores
                    .into_iter()
                    .enumerate()
                    .max_by(|left, right| left.1.total_cmp(&right.1))
                    .ok_or_else(|| Error::custom("ai::invalid", "empty NLI classification output"))?;

                Ok(vec![Annotation {
                    name: String::from("classification"),
                    text: text.to_string(),
                    label: self.labels[index].clone(),
                    score,
                    spans: vec![Offset::new(0, text.chars().count() as u32)],
                }])
            })
            .collect()
    }
}

impl Forward for ZeroShotClassifier {
    type Input = Tensor;
    type Output = Tensor;

    fn forward(&self, input: Self::Input) -> Result<Self::Output, Error> {
        let (batch_size, seq_len) = input.dims2()?;

        if batch_size == 0 || seq_len == 0 {
            return Err(Error::custom("ai::invalid", "empty classification input"));
        }

        let eos = input
            .to_vec2::<u32>()?
            .iter()
            .map(|ids| {
                ids.iter()
                    .rposition(|id| *id == self.eos_token_id)
                    .ok_or_else(|| Error::custom("ai::invalid", "NLI input is missing EOS"))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let start = Tensor::full(self.decoder_start_token_id, (batch_size, 1), input.device())?;
        let shifted = Tensor::cat(&[&start, &input.narrow(1, 0, seq_len - 1)?], 1)?;
        let mask: Vec<f32> = (0..seq_len)
            .flat_map(|i| (0..seq_len).map(move |j| if j > i { f32::NEG_INFINITY } else { 0.0 }))
            .collect();
        let mask = Tensor::from_vec(mask, (seq_len, seq_len), input.device())?;
        let mut model = self
            .model
            .lock()
            .map_err(|_| Error::custom("ai", "NLI model lock poisoned"))?;
        let (encoder, decoder) = &mut *model;
        decoder.reset_kv_cache();
        let encoded = encoder.forward(&input)?;
        let output = decoder.forward(&shifted, Some(&encoded), 0, &mask);
        decoder.reset_kv_cache();
        let output = output?;
        let pooled = eos
            .into_iter()
            .enumerate()
            .map(|(index, eos)| output.i((index, eos)))
            .collect::<Result<Vec<_>, _>>()?;
        let pooled = Tensor::stack(&pooled, 0)?;

        Ok(pooled.apply(&self.dense)?.tanh()?.apply(&self.classifier)?)
    }
}

impl Classify for ZeroShotClassifier {
    fn classify(&self, ctx: &Context, text: &[&str], options: &ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        options.validate()?;

        match (options.mode, options.labels) {
            (ClassifyMode::Text, Some(_)) => Ok(self
                .custom_text(ctx, text, options)?
                .into_iter()
                .map(|annotations| zero_shot::select(annotations, options.multi_label))
                .collect()),
            (ClassifyMode::Text, None) => self.text(ctx, text),
            (ClassifyMode::Spans, Some(_)) => zero_shot::entities(self, ctx, text, options),
            (ClassifyMode::Spans, None) => Err(Error::custom("ai::invalid", "NLI span classification requires custom labels")),
        }
    }
}
