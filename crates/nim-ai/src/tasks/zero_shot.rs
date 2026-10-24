use unicode_categories::UnicodeCategories;

use crate::Error;
use crate::models::{Classify, ClassifyMode, ClassifyOptions, Context};
use crate::types::{Annotation, Offset};

use super::{char_offset, classify};

pub fn softmax(logits: &[f64]) -> Vec<f64> {
    let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let exp: Vec<f64> = logits.iter().map(|logit| (logit - max).exp()).collect();
    let total: f64 = exp.iter().sum();

    exp.into_iter().map(|value| value / total).collect()
}

pub fn select(labels: Vec<Annotation>, multi_label: bool) -> Vec<Annotation> {
    if multi_label {
        return labels;
    }

    labels
        .into_iter()
        .reduce(|left, right| if right.score > left.score { right } else { left })
        .into_iter()
        .collect()
}

pub fn entities<C: Classify + ?Sized>(
    model: &C,
    ctx: &Context,
    text: &[&str],
    options: &ClassifyOptions<'_>,
) -> Result<Vec<Vec<Annotation>>, Error> {
    let options = ClassifyOptions {
        mode: ClassifyMode::Text,
        ..options.clone()
    };

    text.iter()
        .map(|text| {
            let candidates = spans(text);
            let mut entities = Vec::new();

            for chunk in candidates.chunks(32) {
                let input: Vec<&str> = chunk.iter().map(|(start, end)| &text[*start..*end]).collect();
                let scores = classify(model, ctx, &input, &options)?;

                if scores.len() != chunk.len() {
                    return Err(Error::custom("ai::invalid", "NLI output does not match candidate count"));
                }

                for ((start, end), labels) in chunk.iter().zip(scores) {
                    for label in select(labels, options.multi_label) {
                        entities.push(Annotation {
                            name: String::from("entity"),
                            text: text[*start..*end].to_string(),
                            label: label.label,
                            score: label.score,
                            spans: vec![Offset::new(char_offset(text, *start), char_offset(text, *end))],
                        });
                    }
                }
            }

            Ok(entities)
        })
        .collect()
}

pub fn retain(annotations: Vec<Annotation>, options: &ClassifyOptions<'_>, min_score: f64) -> Result<Vec<Annotation>, Error> {
    let labels = options
        .labels
        .ok_or_else(|| Error::custom("ai::config", "custom labels are required"))?;

    if annotations.iter().any(|annotation| {
        !labels.contains(&annotation.label) || !annotation.score.is_finite() || !(0.0..=1.0).contains(&annotation.score)
    }) {
        return Err(Error::custom(
            "ai::invalid",
            "custom classification returned an invalid label or score",
        ));
    }

    let mut annotations: Vec<Annotation> = annotations
        .into_iter()
        .filter(|annotation| annotation.score >= min_score)
        .flat_map(|annotation| {
            annotation
                .spans
                .clone()
                .into_iter()
                .filter(|span| span.begin < span.end)
                .map(move |span| Annotation {
                    spans: vec![span],
                    ..annotation.clone()
                })
        })
        .collect();
    annotations.sort_by(|left, right| {
        let left_span = left.spans[0];
        let right_span = right.spans[0];

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
    let mut retained: Vec<Annotation> = Vec::new();

    for annotation in annotations {
        let span = annotation.spans[0];
        let overlap = retained.iter().any(|other| {
            let other_span = other.spans[0];

            if span == other_span && options.multi_label && annotation.label != other.label {
                return false;
            }

            span.begin < other_span.end && other_span.begin < span.end
        });

        if !overlap {
            retained.push(annotation);
        }
    }

    retained.sort_by(|left, right| {
        left.spans[0].begin.cmp(&right.spans[0].begin).then_with(|| {
            labels
                .iter()
                .position(|label| label == &left.label)
                .cmp(&labels.iter().position(|label| label == &right.label))
        })
    });

    Ok(retained)
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
