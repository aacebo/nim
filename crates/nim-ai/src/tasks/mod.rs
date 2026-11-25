pub mod aggregation;
pub mod anchor;
pub mod bioes;
pub mod zero_shot;

mod candidates;
mod keybert;
mod scorer;
mod stopwords;

use crate::types::{Classification, Keyword, Offset, Polarity, Span, TextOutput, Word};
use crate::{
    Caption, CaptionOptions, Chat, Classify, ClassifyMode, ClassifyOptions, Complete, Context, Embed, EmbedOptions, Summarize,
    SummarizeOptions,
};
#[cfg(feature = "audio")]
use crate::{Transcribe, TranscribeOptions};

/// The user-facing routines, each a presentation over one model capability.
///
/// They are free functions, not traits: a task is something you do *with* a capable model, not a
/// capability a model has. Writing them as blanket impls would also collide -- a hosted model has
/// every capability, so `impl<T: Embed> Keywords for T` conflicts with its own keyword path.
pub async fn embed<E: Embed + ?Sized>(model: &E, ctx: &Context<'_>, input: &[&[u8]]) -> nim_error::Result<Vec<Vec<f32>>> {
    model.embed(ctx, input).await
}

pub async fn embed_with<E: Embed + ?Sized>(
    model: &E,
    ctx: &Context<'_>,
    input: &[&[u8]],
    options: &EmbedOptions,
) -> nim_error::Result<Vec<Vec<f32>>> {
    options.validate()?;
    model.embed_with(ctx, input, options).await
}

/// KeyBERT, over any embedder -- local or hosted. Previously this only worked locally, by
/// special-casing a call into the embeddings pipeline's cache.
pub async fn keywords<E: Embed + ?Sized>(
    model: &E,
    ctx: &Context<'_>,
    input: &[&[u8]],
    top_n: usize,
) -> nim_error::Result<Vec<Vec<Keyword>>> {
    keybert::keywords(model, ctx, input, top_n).await
}

pub async fn classify<C: Classify + ?Sized>(
    model: &C,
    ctx: &Context<'_>,
    input: &[&[u8]],
    options: ClassifyOptions<'_>,
) -> nim_error::Result<Vec<Vec<Classification>>> {
    options.validate()?;
    let out = model.classify(ctx, input, options.clone()).await?;

    if out.len() != input.len() {
        return Err(nim_error::Diagnostic::message(
            "classification output does not match input count",
        ));
    }

    for (classifications, input) in out.iter().zip(input) {
        for classification in classifications {
            if !classification.score.is_finite()
                || !(0.0..=1.0).contains(&classification.score)
                || options.labels.is_some_and(|labels| !labels.contains(&classification.label))
            {
                return Err(nim_error::Diagnostic::message(
                    "classification returned an invalid label or score",
                ));
            }

            for span in &classification.spans {
                match span {
                    Span::Offset(offset) => {
                        let text = span.text(input)?;

                        if options.mode == ClassifyMode::Whole && (offset.begin != 0 || text.as_bytes() != *input) {
                            return Err(nim_error::Diagnostic::message(
                                "whole-text classification must span the full input",
                            ));
                        }
                    }
                    Span::Time(time) => {
                        if !time.begin.is_finite() || !time.end.is_finite() || time.begin < 0.0 || time.begin > time.end {
                            return Err(nim_error::Diagnostic::message("invalid audio time span"));
                        }
                    }
                    Span::Space(bbox) => {
                        if [bbox.xmin, bbox.ymin, bbox.xmax, bbox.ymax]
                            .iter()
                            .any(|value| !value.is_finite() || *value < 0.0)
                            || bbox.xmin > bbox.xmax
                            || bbox.ymin > bbox.ymax
                        {
                            return Err(nim_error::Diagnostic::message("invalid image bounding box"));
                        }
                    }
                }
            }
        }
    }

    Ok(out)
}

pub async fn complete<M: Complete + ?Sized>(
    model: &M,
    ctx: &Context<'_>,
    chat: Chat,
) -> nim_error::Result<nim_chat::ChatAgentMessage> {
    chat.validate()?;
    model.complete(ctx, chat).await
}

pub async fn summarize<M: Summarize + ?Sized>(
    model: &M,
    ctx: &Context<'_>,
    input: &[&[u8]],
    options: SummarizeOptions,
) -> nim_error::Result<Vec<TextOutput>> {
    options.validate()?;
    let output = model.summarize(ctx, input, options).await?;

    if output.len() != input.len() {
        return Err(nim_error::Diagnostic::message("summarize output does not match input count"));
    }

    Ok(output)
}

pub async fn caption<M: Caption + ?Sized>(
    model: &M,
    ctx: &Context<'_>,
    input: &[&[u8]],
    options: CaptionOptions,
) -> nim_error::Result<Vec<TextOutput>> {
    options.validate()?;
    let output = model.caption(ctx, input, options).await?;

    if output.len() != input.len() {
        return Err(nim_error::Diagnostic::message("caption output does not match input count"));
    }

    Ok(output)
}

#[cfg(feature = "audio")]
pub async fn transcribe<M: Transcribe + ?Sized>(
    model: &M,
    ctx: &Context<'_>,
    input: &[&[u8]],
    options: TranscribeOptions<'_>,
) -> nim_error::Result<Vec<TextOutput>> {
    options.validate()?;
    let output = model.transcribe(ctx, input, options).await?;

    if output.len() != input.len() {
        return Err(nim_error::Diagnostic::message("transcribe output does not match input count"));
    }

    Ok(output)
}

pub(crate) fn classification(labels: &[String], index: usize, score: f64, text: &str) -> Classification {
    let sentiment = labels.len() == 2
        && labels
            .iter()
            .any(|label| matches!(label.to_lowercase().as_str(), "positive" | "pos"))
        && labels
            .iter()
            .any(|label| matches!(label.to_lowercase().as_str(), "negative" | "neg"));
    let label = &labels[index];

    Classification {
        name: String::from(if sentiment { "sentiment" } else { "classification" }),
        label: if sentiment {
            Polarity::from_label(label).as_str().to_string()
        } else {
            label.clone()
        },
        score,
        spans: vec![Span::Offset(Offset::new(0, text.chars().count() as u32))],
    }
}

pub(crate) fn decode_entities(words: Vec<Word>, text: &str, labels: &[String]) -> Vec<Classification> {
    match labels.iter().any(|label| label.starts_with("E-") || label.starts_with("S-")) {
        true => bioes::entities(words, text, 0.0),
        false => aggregation::entities(words, text),
    }
}

/// Char offsets, not byte offsets: a span is reported in the units the caller sees.
pub(crate) fn char_offset(text: &str, byte_offset: usize) -> u32 {
    let byte_offset = byte_offset.min(text.len());
    text.char_indices().take_while(|(index, _)| *index < byte_offset).count() as u32
}

pub(crate) fn response_text(content: Option<nim_chat::ChatMessageContent>) -> String {
    match content {
        Some(nim_chat::ChatMessageContent::String(text)) => text,
        Some(nim_chat::ChatMessageContent::Parts(parts)) => parts
            .into_iter()
            .filter_map(|part| match part {
                nim_chat::ChatMessageContentPart::Text { text } => Some(text),
                nim_chat::ChatMessageContentPart::Url { url } => Some(url.to_string()),
                nim_chat::ChatMessageContentPart::Json { json } => Some(json.to_string()),
                nim_chat::ChatMessageContentPart::Speak { speak } => Some(speak),
                _ => None,
            })
            .collect(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod capability_tests {
    use super::*;
    use candle_core::Device;

    struct TestModel;

    impl Complete for TestModel {
        fn complete<'a>(
            &'a self,
            _: &'a Context<'_>,
            _: Chat,
        ) -> futures_util::future::BoxFuture<'a, nim_error::Result<nim_chat::ChatAgentMessage>> {
            Box::pin(async {
                Ok(nim_chat::ChatAgentMessage {
                    name: None,
                    content: Some(nim_chat::ChatMessageContent::String("reply".into())),
                    tool_calls: Vec::new(),
                    annotations: Vec::new(),
                    extra: Default::default(),
                })
            })
        }
    }

    impl Summarize for TestModel {
        fn summarize<'a>(
            &'a self,
            _: &'a Context<'_>,
            _: &'a [&'a [u8]],
            _: SummarizeOptions,
        ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    #[tokio::test]
    async fn complete_and_summarize_validate_options_and_batch_size() {
        let device = Device::Cpu;
        let ctx = Context::new(None, &device, "test");
        let input = [b"input".as_slice()];
        let options = Chat::default();
        assert!(complete(&TestModel, &ctx, options).await.is_ok());
        assert!(
            complete(
                &TestModel,
                &ctx,
                Chat {
                    max_tokens: Some(0),
                    ..Default::default()
                }
            )
            .await
            .is_err()
        );
        assert!(
            summarize(&TestModel, &ctx, &input, SummarizeOptions::default())
                .await
                .is_err()
        );
    }
}
