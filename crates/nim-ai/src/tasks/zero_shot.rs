use unicode_categories::UnicodeCategories;

use crate::types::{Classification, Offset, Span};
use crate::{Classify, ClassifyMode, ClassifyOptions, Context};

use super::{char_offset, classify};

pub fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exp: Vec<f64> = logits.iter().map(|logit| (logit - max).exp()).collect();
    let total: f64 = exp.iter().sum();

    exp.into_iter().map(|value| value / total).collect()
}

pub fn select(labels: Vec<Classification>, multi_label: bool) -> Vec<Classification> {
    if multi_label {
        return labels;
    }

    labels
        .into_iter()
        .reduce(|left, right| if right.score > left.score { right } else { left })
        .into_iter()
        .collect()
}

pub async fn entities<C: Classify + ?Sized>(
    model: &C,
    ctx: &Context<'_>,
    input: &[&[u8]],
    options: ClassifyOptions<'_>,
) -> nim_error::Result<Vec<Vec<Classification>>> {
    let options = ClassifyOptions {
        mode: ClassifyMode::Whole,
        min_score: 0.0,
        limit: None,
        ..options.clone()
    };

    let text = input
        .iter()
        .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
        .collect::<Result<Vec<_>, _>>()?;

    let mut output = Vec::with_capacity(text.len());

    for text in text {
        let candidates = spans(text);
        let mut entities = Vec::new();

        for chunk in candidates.chunks(32) {
            let input: Vec<&[u8]> = chunk.iter().map(|(start, end)| &text.as_bytes()[*start..*end]).collect();
            let scores = classify(model, ctx, &input, options.clone()).await?;

            if scores.len() != chunk.len() {
                return Err(nim_error::Diagnostic::message("NLI output does not match candidate count"));
            }

            for ((start, end), labels) in chunk.iter().zip(scores) {
                for label in select(labels, options.multi_label) {
                    entities.push(Classification {
                        name: String::from("entity"),

                        label: label.label,
                        score: label.score,
                        spans: vec![Span::Offset(Offset::new(char_offset(text, *start), char_offset(text, *end)))],
                    });
                }
            }
        }

        output.push(entities);
    }

    Ok(output)
}

pub fn retain(
    classifications: Vec<Classification>,
    options: ClassifyOptions<'_>,
    min_score: f64,
) -> nim_error::Result<Vec<Classification>> {
    let labels = options
        .labels
        .ok_or_else(|| nim_error::Diagnostic::message("custom labels are required"))?;

    if classifications.iter().any(|classification| {
        !labels.contains(&classification.label)
            || !classification.score.is_finite()
            || !(0.0..=1.0).contains(&classification.score)
    }) {
        return Err(nim_error::Diagnostic::message(
            "custom classification returned an invalid label or score",
        ));
    }

    let mut candidates = Vec::new();

    for classification in classifications
        .into_iter()
        .filter(|classification| classification.score >= min_score)
    {
        for span in &classification.spans {
            let Span::Offset(offset) = span else {
                return Err(nim_error::Diagnostic::message("text extraction requires text spans"));
            };

            if offset.begin < offset.end {
                candidates.push((
                    *offset,
                    Classification {
                        spans: vec![*span],
                        ..classification.clone()
                    },
                ));
            }
        }
    }

    candidates.sort_by(|(left_span, left), (right_span, right)| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| (right_span.end - right_span.begin).cmp(&(left_span.end - left_span.begin)))
            .then_with(|| left_span.begin.cmp(&right_span.begin))
            .then_with(|| {
                labels
                    .iter()
                    .position(|label| label == &left.label)
                    .cmp(&labels.iter().position(|label| label == &right.label))
            })
    });
    let mut retained: Vec<(Offset, Classification)> = Vec::new();

    for (span, classification) in candidates {
        let overlap = retained.iter().any(|(other_span, other)| {
            if span == *other_span && options.multi_label && classification.label != other.label {
                return false;
            }

            span.begin < other_span.end && other_span.begin < span.end
        });

        if !overlap {
            retained.push((span, classification));
        }
    }

    retained.sort_by(|(left_span, left), (right_span, right)| {
        left_span.begin.cmp(&right_span.begin).then_with(|| {
            labels
                .iter()
                .position(|label| label == &left.label)
                .cmp(&labels.iter().position(|label| label == &right.label))
        })
    });

    Ok(retained.into_iter().map(|(_, classification)| classification).collect())
}

fn spans(text: &str) -> Vec<(usize, usize)> {
    let mut sentences: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut sentence = Vec::new();
    let mut cursor = 0;

    for word in text.split_whitespace() {
        let start = cursor + text[cursor..].find(word).unwrap_or(0);
        let end = start + word.len();

        if text[cursor..start].contains(['\n', '\r']) && !sentence.is_empty() {
            sentences.push(std::mem::take(&mut sentence));
        }

        let trimmed = word.trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_punctuation());

        if !trimmed.is_empty() {
            let begin = start + word.find(trimmed).unwrap_or(0);
            sentence.push((begin, begin + trimmed.len()));
        }

        if word
            .trim_end_matches(|ch: char| ch.is_punctuation_close() || ch.is_punctuation_final_quote() || matches!(ch, '\'' | '"'))
            .ends_with(['.', '!', '?', '。', '！', '？'])
            && !sentence.is_empty()
        {
            sentences.push(std::mem::take(&mut sentence));
        }

        cursor = end;
    }

    if !sentence.is_empty() {
        sentences.push(sentence);
    }

    let mut spans = Vec::new();

    for sentence in sentences {
        for start in 0..sentence.len() {
            for end in start..sentence.len().min(start + 6) {
                spans.push((sentence[start].0, sentence[end].1));
            }
        }
    }

    spans
}
