use crate::Error;
use crate::models::{Context, Embed};
use crate::pipelines::common::decode_text;
use crate::types::Keyword;

use super::{candidates, scorer};

/// KeyBERT: embed the document, embed each candidate phrase, rank candidates by cosine similarity
/// to the document. Any embedder will do -- which is the point of taking `&dyn Embed` rather than
/// a concrete model.
pub fn keywords<E: Embed + ?Sized>(model: &E, ctx: &Context, input: &[&[u8]], top_n: usize) -> Result<Vec<Vec<Keyword>>, Error> {
    let text = decode_text(input)?;
    text.iter().map(|text| one(model, ctx, text, top_n)).collect()
}

fn one<E: Embed + ?Sized>(model: &E, ctx: &Context, text: &str, top_n: usize) -> Result<Vec<Keyword>, Error> {
    let candidates = candidates::extract(text);

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // The document and its candidates go through in one batch, so the document vector and the
    // candidate vectors come from the same forward pass.
    let mut batch: Vec<&[u8]> = Vec::with_capacity(candidates.len() + 1);
    batch.push(text.as_bytes());
    batch.extend(candidates.iter().map(|candidate| candidate.text.as_bytes()));

    let vectors = model.embed(ctx, &batch)?;
    if vectors.len() != batch.len() {
        return Err(Error::custom("ai::invalid", "keyword embeddings do not match input count"));
    }

    let document = &vectors[0];

    if document.is_empty()
        || vectors
            .iter()
            .any(|vector| vector.len() != document.len() || vector.iter().any(|value| !value.is_finite()))
    {
        return Err(Error::custom("ai::invalid", "invalid keyword embeddings"));
    }

    let mut keywords = scorer::rank(candidates, document, &vectors[1..], usize::MAX);

    for keyword in &mut keywords {
        if !keyword.score.is_finite() || !(-1.000001..=1.000001).contains(&keyword.score) {
            return Err(Error::custom("ai::invalid", "invalid keyword cosine score"));
        }

        keyword.score = keyword.score.clamp(-1.0, 1.0);
    }

    keywords.truncate(top_n);
    Ok(keywords)
}
