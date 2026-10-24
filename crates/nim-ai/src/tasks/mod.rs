pub mod aggregation;
pub mod anchor;
pub mod bioes;
pub mod zero_shot;

mod candidates;
mod keybert;
mod scorer;
mod stopwords;

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context, Embed, GenOpts, Generate, Word};
use crate::types::{Annotation, Keyword, Polarity, Sentiment};

/// The user-facing routines, each a presentation over one model capability.
///
/// They are free functions, not traits: a task is something you do *with* a capable model, not a
/// capability a model has. Writing them as blanket impls would also collide -- a hosted model has
/// every capability, so `impl<T: Embed> Keywords for T` conflicts with its own keyword path.
pub fn embed<E: Embed + ?Sized>(model: &E, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<f32>>, Error> {
    model.embed(ctx, text)
}

/// KeyBERT, over any embedder -- local or hosted. Previously this only worked locally, by
/// special-casing a call into the embeddings pipeline's cache.
pub fn keywords<E: Embed + ?Sized>(model: &E, ctx: &Context, text: &[&str], top_n: usize) -> Result<Vec<Vec<Keyword>>, Error> {
    keybert::keywords(model, ctx, text, top_n)
}

/// Sentiment is a *presentation* of sequence classification: the model returns a label, and this
/// maps it onto the binary polarity the routine promises.
pub fn sentiment<C: Classify + ?Sized>(model: &C, ctx: &Context, text: &[&str]) -> Result<Vec<Sentiment>, Error> {
    classify(model, ctx, text, &Default::default())?
        .into_iter()
        .map(|annotations| {
            let annotation = annotations
                .into_iter()
                .next()
                .ok_or_else(|| Error::custom("ai::invalid", "empty sentiment classification output"))?;

            Ok(Sentiment {
                polarity: Polarity::from_label(&annotation.label),
                score: annotation.score,
            })
        })
        .collect()
}

pub fn classify<C: Classify + ?Sized>(
    model: &C,
    ctx: &Context,
    text: &[&str],
    options: &ClassifyOptions<'_>,
) -> Result<Vec<Vec<Annotation>>, Error> {
    options.validate()?;
    let out = model.classify(ctx, text, options)?;

    if out.len() != text.len() {
        return Err(Error::custom(
            "ai::invalid",
            "classification output does not match input count",
        ));
    }

    for (annotations, text) in out.iter().zip(text) {
        if options.mode == ClassifyMode::Text && !options.multi_label && annotations.len() != 1 {
            return Err(Error::custom(
                "ai::invalid",
                "single-label classification must return one annotation",
            ));
        }

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

            if annotation.spans.is_empty()
                || annotation.spans.iter().any(|span| {
                    span.begin > span.end
                        || span.end as usize > text.chars().count()
                        || text
                            .chars()
                            .skip(span.begin as usize)
                            .take((span.end - span.begin) as usize)
                            .collect::<String>()
                            != annotation.text
                })
            {
                return Err(Error::custom(
                    "ai::invalid",
                    "classification text does not match its source spans",
                ));
            }

            if options.mode == ClassifyMode::Text
                && annotation.spans != [crate::types::Offset::new(0, text.chars().count() as u32)]
            {
                return Err(Error::custom("ai::invalid", "text classification must span the full input"));
            }
        }
    }

    Ok(out)
}

pub fn entities<C: Classify + ?Sized>(model: &C, ctx: &Context, text: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
    classify(
        model,
        ctx,
        text,
        &ClassifyOptions {
            mode: ClassifyMode::Spans,
            ..Default::default()
        },
    )
}

pub(crate) fn decode_entities(words: Vec<Word>, text: &str, labels: &[String]) -> Vec<Annotation> {
    match labels.iter().any(|label| label.starts_with("E-") || label.starts_with("S-")) {
        true => bioes::entities(words, text, 0.0),
        false => aggregation::entities(words, text),
    }
}

pub fn summarize<G: Generate + ?Sized>(model: &G, ctx: &Context, text: &[&str], opts: &GenOpts) -> Result<Vec<String>, Error> {
    model.generate(ctx, text, opts)
}

/// Char offsets, not byte offsets: a span is reported in the units the caller sees.
pub(crate) fn char_offset(text: &str, byte_offset: usize) -> u32 {
    let byte_offset = byte_offset.min(text.len());
    text.char_indices().take_while(|(index, _)| *index < byte_offset).count() as u32
}
