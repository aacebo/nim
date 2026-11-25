use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Embed};
use crate::tasks;
use crate::types::{Classification, Span};

pub struct KeyBert<E: Embed> {
    embedder: E,
}

impl<E: Embed> KeyBert<E> {
    pub fn new(embedder: E) -> Self {
        Self { embedder }
    }
}

impl<E: Embed> Embed for KeyBert<E> {
    fn embed<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        Box::pin(async move { self.embedder.embed(ctx, input).await })
    }
}

impl<E: Embed> Classify for KeyBert<E> {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        Box::pin(async move {
            let text = input
                .iter()
                .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
                .collect::<Result<Vec<_>, _>>()?;
            options.validate()?;

            if options.mode != ClassifyMode::Regions
                || options.labels.is_some()
                || options.multi_label
                || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
            {
                return Err(nim_error::Diagnostic::message(
                    "KeyBERT only supports keyword regions without custom labels",
                ));
            }

            if options.limit == Some(0) {
                return Ok(vec![Vec::new(); input.len()]);
            }

            let out = tasks::keywords(self, ctx, input, usize::MAX).await?;
            out.into_iter()
                .zip(text)
                .map(|(keywords, text)| {
                    let mut classifications: Vec<(String, Classification)> = Vec::new();

                    for keyword in keywords
                        .into_iter()
                        .filter(|keyword| keyword.score as f64 >= options.min_score)
                    {
                        for offset in keyword.offsets {
                            let span = Span::Offset(offset);
                            let source = span.text(text.as_bytes())?;

                            if let Some((_, classification)) = classifications.iter_mut().find(|(source_text, classification)| {
                                source_text == source && classification.label == keyword.text
                            }) {
                                classification.spans.push(span);
                            } else {
                                classifications.push((
                                    source.to_string(),
                                    Classification {
                                        name: String::from("keyword"),
                                        label: keyword.text.clone(),
                                        score: keyword.score as f64,
                                        spans: vec![span],
                                    },
                                ));
                            }
                        }
                    }

                    options.apply(
                        classifications
                            .into_iter()
                            .map(|(_, classification)| classification)
                            .collect(),
                    )
                })
                .collect()
        })
    }
}
