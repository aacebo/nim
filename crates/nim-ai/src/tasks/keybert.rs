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
    let Some((document, vectors)) = vectors.split_first() else {
        return Ok(Vec::new());
    };

    Ok(scorer::rank(candidates, document, vectors, top_n))
}
