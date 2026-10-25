use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Embed};
use crate::pipelines::common::decode_text;
use crate::tasks;
use crate::types::{Annotation, Span};

pub struct KeyBert<E: Embed> {
    embedder: E,
}

impl<E: Embed> KeyBert<E> {
    pub fn new(embedder: E) -> Self {
        Self { embedder }
    }
}

impl<E: Embed> Embed for KeyBert<E> {
    fn embed(&self, ctx: &Context, input: &[&[u8]]) -> Result<Vec<Vec<f32>>, Error> {
        self.embedder.embed(ctx, input)
    }
}

impl<E: Embed> Classify for KeyBert<E> {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        let text = decode_text(input)?;
        options.validate()?;

        if options.mode != ClassifyMode::Regions
            || options.labels.is_some()
            || options.multi_label
            || options.hypothesis_template != ClassifyOptions::default().hypothesis_template
        {
            return Err(Error::custom(
                "ai::config",
                "KeyBERT only supports keyword regions without custom labels",
            ));
        }

        if options.limit == Some(0) {
            return Ok(vec![Vec::new(); input.len()]);
        }

        let out = tasks::keywords(self, ctx, input, usize::MAX)?;
        out.into_iter()
            .zip(text)
            .map(|(keywords, text)| {
                let mut annotations: Vec<(String, Annotation)> = Vec::new();

                for keyword in keywords
                    .into_iter()
                    .filter(|keyword| keyword.score as f64 >= options.min_score)
                {
                    for offset in keyword.offsets {
                        let span = Span::Text(offset);
                        let source = span
                            .text(text.as_bytes())?
                            .ok_or_else(|| Error::custom("ai::invalid", "keyword requires a text span"))?;

                        if let Some((_, annotation)) = annotations
                            .iter_mut()
                            .find(|(source_text, annotation)| source_text == source && annotation.label == keyword.text)
                        {
                            annotation.spans.push(span);
                        } else {
                            annotations.push((
                                source.to_string(),
                                Annotation {
                                    name: String::from("keyword"),
                                    label: keyword.text.clone(),
                                    score: keyword.score as f64,
                                    spans: vec![span],
                                },
                            ));
                        }
                    }
                }

                options.apply(annotations.into_iter().map(|(_, annotation)| annotation).collect())
            })
            .collect()
    }
}
