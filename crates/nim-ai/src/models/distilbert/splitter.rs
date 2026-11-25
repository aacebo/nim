use std::collections::BTreeMap;

use candle_core::Tensor;
use candle_nn::{VarBuilder, ops};
use tokenizers::TruncationParams;

use crate::tasks::aggregation;
use crate::types::{TextOutput, Word};
use crate::{ClassifyMode, Context, Split, SplitOptions, models::Forward};

use super::{DistilBertClassifier, DistilBertConfig};

pub struct DistilBertSplitter {
    classifier: DistilBertClassifier,
    max_positions: usize,
}

impl DistilBertSplitter {
    pub fn new(vars: VarBuilder, config: &DistilBertConfig) -> nim_error::Result<Self> {
        Ok(Self {
            classifier: DistilBertClassifier::new(vars, config, ClassifyMode::Regions)?,
            max_positions: config.max_position_embeddings,
        })
    }
}

impl Split for DistilBertSplitter {
    fn split<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a str,
        options: SplitOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            let delimiters = self.split_delimited(ctx, input, options).await?;
            let mut start = 0;
            let mut output = Vec::with_capacity(delimiters.len() + 1);

            for delimiter in delimiters {
                output.push(TextOutput {
                    text: input[start..delimiter.end].to_string(),
                    confidence: None,
                });
                start = delimiter.end;
            }

            if start < input.len() {
                output.push(TextOutput {
                    text: input[start..].to_string(),
                    confidence: None,
                });
            }

            Ok(output)
        })
    }

    fn split_delimited<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a str,
        options: SplitOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Word>>> {
        Box::pin(async move {
            if options.max_length == Some(0) {
                return Err(nim_error::Diagnostic::message("max_length must be greater than zero"));
            }

            if input.is_empty() || options.max_length == Some(1) {
                return Ok(Vec::new());
            }

            if self.max_positions <= 2 {
                return Err(nim_error::Diagnostic::message("split model context is too small"));
            }

            let mut tokenizer = ctx.tokenizer()?.clone();
            tokenizer
                .with_truncation(Some(TruncationParams {
                    max_length: self.max_positions,
                    stride: (self.max_positions - 2) / 2,
                    ..Default::default()
                }))
                .map_err(nim_error::Diagnostic::message)?;
            let encoding = tokenizer.encode(input, true).map_err(nim_error::Diagnostic::message)?;
            let labels = [String::from("O"), String::from("separator")];
            let mut selected = BTreeMap::new();

            for window in std::iter::once(&encoding).chain(encoding.get_overflowing()) {
                let ids = window.get_ids();
                let shape = (1, ids.len());
                let ids = Tensor::from_slice(ids, shape, ctx.device())?;
                let mask = Tensor::from_slice(window.get_attention_mask(), shape, ctx.device())?;
                let padding = (mask.ones_like()? - mask)?;
                let logits = self.classifier.forward((ids, padding))?;
                let probabilities = ops::softmax(&logits, 2)?.squeeze(0)?.to_vec2::<f32>()?;
                let words = aggregation::words(&probabilities, window, &labels)?;
                let mut previous = None;
                let ids = window.get_word_ids().iter().filter_map(|id| {
                    let id = (*id)?;
                    let duplicate = previous == Some(id);
                    previous = Some(id);
                    (!duplicate).then_some(id)
                });

                for (id, word) in ids.zip(words) {
                    let replace = selected.get(&id).is_none_or(|old: &Word| {
                        word.end - word.start > old.end - old.start
                            || word.end - word.start == old.end - old.start && word.score > old.score
                    });

                    if replace {
                        selected.insert(id, word);
                    }
                }
            }

            let mut delimiters: Vec<Word> = Vec::new();
            let mut previous_separator = false;
            let mut count = 0;

            for word in selected.into_values() {
                let separator = word.label == "separator";

                if separator && previous_separator {
                    if let Some(previous) = delimiters.last_mut() {
                        previous.end = word.end;
                        previous.score = (previous.score * count as f64 + word.score) / (count + 1) as f64;
                        count += 1;
                    }
                } else if separator {
                    delimiters.push(word);
                    count = 1;
                }

                previous_separator = separator;
            }

            delimiters.retain(|word| word.end > 0 && word.end < input.len());

            if let Some(max_length) = options.max_length {
                delimiters.truncate(max_length - 1);
            }

            Ok(delimiters)
        })
    }
}
