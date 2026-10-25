pub mod aggregation;
pub mod anchor;
pub mod bioes;
pub mod zero_shot;

mod candidates;
mod keybert;
mod scorer;
mod stopwords;

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Embed, Generate, GenerateOptions, Word};
use crate::types::{Annotation, Keyword, Offset, Polarity, Span};

/// The user-facing routines, each a presentation over one model capability.
///
/// They are free functions, not traits: a task is something you do *with* a capable model, not a
/// capability a model has. Writing them as blanket impls would also collide -- a hosted model has
/// every capability, so `impl<T: Embed> Keywords for T` conflicts with its own keyword path.
pub fn embed<E: Embed + ?Sized>(model: &E, ctx: &Context, input: &[&[u8]]) -> Result<Vec<Vec<f32>>, Error> {
    model.embed(ctx, input)
}

/// KeyBERT, over any embedder -- local or hosted. Previously this only worked locally, by
/// special-casing a call into the embeddings pipeline's cache.
pub fn keywords<E: Embed + ?Sized>(model: &E, ctx: &Context, input: &[&[u8]], top_n: usize) -> Result<Vec<Vec<Keyword>>, Error> {
    keybert::keywords(model, ctx, input, top_n)
}

pub fn classify<C: Classify + ?Sized>(
    model: &C,
    ctx: &Context,
    input: &[&[u8]],
    options: ClassifyOptions<'_>,
) -> Result<Vec<Vec<Annotation>>, Error> {
    options.validate()?;
    let out = model.classify(ctx, input, options.clone())?;

    if out.len() != input.len() {
        return Err(Error::custom(
            "ai::invalid",
            "classification output does not match input count",
        ));
    }

    for (annotations, input) in out.iter().zip(input) {
        for annotation in annotations {
            if !annotation.score.is_finite()
                || !(0.0..=1.0).contains(&annotation.score)
                || options.labels.is_some_and(|labels| !labels.contains(&annotation.label))
            {
                return Err(Error::custom(
                    "ai::invalid",
                    "classification returned an invalid label or score",
                ));
            }

            for span in &annotation.spans {
                match span {
                    Span::Text(offset) => {
                        let text = span
                            .text(input)?
                            .ok_or_else(|| Error::custom("ai::invalid", "missing text span"))?;

                        if options.mode == ClassifyMode::Whole && (offset.begin != 0 || text.as_bytes() != *input) {
                            return Err(Error::custom(
                                "ai::invalid",
                                "whole-text classification must span the full input",
                            ));
                        }
                    }
                    Span::Audio(time) => {
                        if !time.begin.is_finite() || !time.end.is_finite() || time.begin < 0.0 || time.begin > time.end {
                            return Err(Error::custom("ai::invalid", "invalid audio time span"));
                        }
                    }
                    Span::Image(bbox) => {
                        if [bbox.xmin, bbox.ymin, bbox.xmax, bbox.ymax]
                            .iter()
                            .any(|value| !value.is_finite() || *value < 0.0)
                            || bbox.xmin > bbox.xmax
                            || bbox.ymin > bbox.ymax
                        {
                            return Err(Error::custom("ai::invalid", "invalid image bounding box"));
                        }
                    }
                }
            }
        }
    }

    Ok(out)
}

pub(crate) fn classification(labels: &[String], index: usize, score: f64, text: &str) -> Annotation {
    let sentiment = labels.len() == 2
        && labels
            .iter()
            .any(|label| matches!(label.to_lowercase().as_str(), "positive" | "pos"))
        && labels
            .iter()
            .any(|label| matches!(label.to_lowercase().as_str(), "negative" | "neg"));
    let label = &labels[index];

    Annotation {
        name: String::from(if sentiment { "sentiment" } else { "classification" }),
        label: if sentiment {
            Polarity::from_label(label).as_str().to_string()
        } else {
            label.clone()
        },
        score,
        spans: vec![Span::Text(Offset::new(0, text.chars().count() as u32))],
    }
}

pub(crate) fn decode_entities(words: Vec<Word>, text: &str, labels: &[String]) -> Vec<Annotation> {
    match labels.iter().any(|label| label.starts_with("E-") || label.starts_with("S-")) {
        true => bioes::entities(words, text, 0.0),
        false => aggregation::entities(words, text),
    }
}

pub fn generate<G: Generate + ?Sized>(
    model: &G,
    ctx: &Context,
    input: &[&[u8]],
    options: GenerateOptions<'_>,
) -> Result<Vec<String>, Error> {
    options.validate()?;
    let out = model.generate(ctx, input, options)?;

    if out.len() != input.len() {
        return Err(Error::custom("ai::invalid", "generation output does not match input count"));
    }

    Ok(out)
}

pub fn summarize<G: Generate + ?Sized>(
    model: &G,
    ctx: &Context,
    input: &[&[u8]],
    options: GenerateOptions<'_>,
) -> Result<Vec<String>, Error> {
    generate(model, ctx, input, options)
}

/// Char offsets, not byte offsets: a span is reported in the units the caller sees.
pub(crate) fn char_offset(text: &str, byte_offset: usize) -> u32 {
    let byte_offset = byte_offset.min(text.len());
    text.char_indices().take_while(|(index, _)| *index < byte_offset).count() as u32
}
