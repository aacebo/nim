use crate::Error;
use crate::pipelines::common::decode_text;
use crate::tasks::{anchor, zero_shot};
use crate::types::{Annotation, Offset, Span};

use super::RemoteModel;
use super::capability::{Classify, ClassifyMode, ClassifyOptions, Context, Embed, Generate, GenerateOptions};

const CLASSIFY: &str = "Classify the text the user gives you. Respond with JSON matching the \
                        schema: `label` is your chosen class and `score` is your confidence from \
                        0.0 to 1.0. Prefer the labels \"positive\" and \"negative\" unless the text \
                        clearly calls for another.";

const TOKENS: &str = "Extract the named entities and personally identifying information from the \
                      text the user gives you. Respond with JSON matching the schema. `label` names \
                      the kind (person, organization, location, misc, email, phone, address, ssn, \
                      credit_card, ...). `text` must be copied verbatim from the input. `score` is \
                      your confidence from 0.0 to 1.0.";

#[derive(serde::Deserialize)]
struct Classified {
    label: String,
    score: f64,
}

#[derive(serde::Deserialize)]
struct Scores {
    labels: Vec<Classified>,
}

#[derive(serde::Deserialize)]
struct Found {
    entities: Vec<Item>,
}

#[derive(serde::Deserialize)]
struct Item {
    label: String,
    text: String,
    score: f64,
}

/// A hosted model serves every capability: the annotation ones through a JSON-schema-constrained
/// chat completion, generation through a plain completion, and embeddings through their own
/// endpoint. This is the full remote row of the capability matrix.
impl Embed for RemoteModel {
    fn embed(&self, _ctx: &Context, input: &[&[u8]]) -> Result<Vec<Vec<f32>>, Error> {
        let text = decode_text(input)?;
        if text.is_empty() {
            return Ok(Vec::new());
        }

        self.client().embeddings(&text)
    }
}

impl Classify for RemoteModel {
    fn classify(&self, _ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        let text = decode_text(input)?;
        options.validate()?;

        let out = match (options.mode, options.labels) {
            (ClassifyMode::Whole, Some(_)) => self.custom_text(&text, options.clone()),
            (ClassifyMode::Whole, None) => self.text(&text),
            (ClassifyMode::Regions, Some(_)) => self.custom_spans(&text, options.clone()),
            (ClassifyMode::Regions, None) => text.iter().map(|text| self.spans(text)).collect(),
        }?;

        out.into_iter()
            .map(|annotations| {
                let annotations = if options.mode == ClassifyMode::Regions && options.labels.is_some() {
                    zero_shot::retain(annotations, options.clone(), options.min_score)?
                } else {
                    annotations
                };
                options.apply(annotations)
            })
            .collect()
    }
}

impl RemoteModel {
    fn text(&self, text: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
        text.iter()
            .map(|text| {
                let out: Classified = self.client().json(CLASSIFY, text, classify_schema())?;

                Ok(vec![Annotation {
                    name: String::from("classification"),

                    label: out.label.to_lowercase(),
                    score: out.score,
                    spans: vec![Span::Text(Offset::new(0, text.chars().count() as u32))],
                }])
            })
            .collect()
    }

    fn custom_text(&self, text: &[&str], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        let candidates = options
            .labels
            .ok_or_else(|| Error::custom("ai::config", "custom labels are required"))?;
        let hypotheses: Vec<String> = candidates
            .iter()
            .map(|label| options.hypothesis_template.replace("{}", label))
            .collect();
        let prompt = format!(
            "Classify the input using exactly these labels: {}. Their hypotheses, in the same order, are: {}. \
             Return every label exactly once with a finite confidence from 0 to 1. {} Preserve label spelling exactly.",
            serde_json::to_string(candidates)?,
            serde_json::to_string(&hypotheses)?,
            if options.multi_label || candidates.len() == 1 {
                "Evaluate each hypothesis independently."
            } else {
                "The labels are mutually exclusive; their scores must sum to 1."
            },
        );
        let mut item = classify_schema();
        item["properties"]["label"]["enum"] = serde_json::json!(candidates);
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "labels": { "type": "array", "items": item } },
            "required": ["labels"],
            "additionalProperties": false,
        });

        text.iter()
            .map(|text| {
                let out: Scores = self.client().json(&prompt, text, schema.clone())?;

                if out.labels.len() != candidates.len()
                    || out.labels.iter().any(|label| {
                        !candidates.contains(&label.label) || !label.score.is_finite() || !(0.0..=1.0).contains(&label.score)
                    })
                {
                    return Err(Error::custom(
                        "ai::invalid",
                        "hosted classification returned invalid labels or scores",
                    ));
                }

                let mut labels = Vec::with_capacity(candidates.len());

                for label in candidates {
                    let matching: Vec<&Classified> = out.labels.iter().filter(|item| &item.label == label).collect();

                    if matching.len() != 1 {
                        return Err(Error::custom(
                            "ai::invalid",
                            "hosted classification must return each label exactly once",
                        ));
                    }

                    labels.push(Annotation {
                        name: String::from("classification"),

                        label: label.clone(),
                        score: matching[0].score,
                        spans: vec![Span::Text(Offset::new(0, text.chars().count() as u32))],
                    });
                }

                if !options.multi_label && candidates.len() > 1 {
                    let total: f64 = labels.iter().map(|label| label.score).sum();

                    if total <= 0.0 {
                        return Err(Error::custom(
                            "ai::invalid",
                            "hosted classification returned zero total confidence",
                        ));
                    }

                    for label in &mut labels {
                        label.score /= total;
                    }
                }

                Ok(zero_shot::select(labels, options.multi_label))
            })
            .collect()
    }

    fn custom_spans(&self, text: &[&str], options: ClassifyOptions<'_>) -> Result<Vec<Vec<Annotation>>, Error> {
        let candidates = options
            .labels
            .ok_or_else(|| Error::custom("ai::config", "custom labels are required"))?;
        let prompt = format!(
            "Extract verbatim spans from the input using only these categories: {}. \
             Interpret each category with this hypothesis template: {}. \
             Preserve category spelling exactly. Return confidence scores from 0 to 1. {}",
            serde_json::to_string(candidates)?,
            options.hypothesis_template,
            if options.multi_label {
                "A span may have multiple categories."
            } else {
                "Assign at most one category per span."
            },
        );
        let mut schema = tokens_schema();
        schema["properties"]["entities"]["items"]["properties"]["label"]["enum"] = serde_json::json!(candidates);

        text.iter()
            .map(|text| {
                let found: Found = self.client().json(&prompt, text, schema.clone())?;
                let mut entities = Vec::new();

                for item in found.entities {
                    if !candidates.contains(&item.label) || !item.score.is_finite() || !(0.0..=1.0).contains(&item.score) {
                        return Err(Error::custom(
                            "ai::invalid",
                            "hosted extraction returned an invalid label or score",
                        ));
                    }

                    for offset in anchor::all(text, &item.text) {
                        entities.push(Annotation {
                            name: String::from("entity"),
                            spans: vec![Span::Text(offset)],

                            label: item.label.clone(),
                            score: item.score,
                        });
                    }
                }

                Ok(entities)
            })
            .collect()
    }

    fn spans(&self, text: &str) -> Result<Vec<Annotation>, Error> {
        let found: Found = self.client().json(TOKENS, text, tokens_schema())?;

        Ok(found
            .entities
            .into_iter()
            // A hosted API gives no offsets, so spans are re-anchored in the source. An entity
            // that is not verbatim cannot be anchored -- and violates the prompt -- so it is
            // dropped rather than given an invented span.
            .filter_map(|item| {
                Some(Annotation {
                    name: String::from("entity"),
                    spans: vec![Span::Text(anchor::find(text, &item.text)?)],

                    label: item.label.to_lowercase(),
                    score: item.score,
                })
            })
            .collect())
    }
}

/// Generation is a plain completion, not a schema-constrained one: the output is prose, and
/// wrapping it in a JSON envelope buys nothing.
impl Generate for RemoteModel {
    fn generate(&self, _ctx: &Context, input: &[&[u8]], options: GenerateOptions<'_>) -> Result<Vec<String>, Error> {
        options.validate()?;

        if options.language.is_some() || options.beams.is_some() {
            return Err(Error::custom(
                "ai::config",
                "hosted text generation does not support language or beams",
            ));
        }

        let text = decode_text(input)?;
        let prompt = options.prompt.unwrap_or(
            "Summarize the text the user gives you. Be concise and factual; use only information present in the text.",
        );
        text.iter()
            .map(|text| self.client().complete(prompt, text, options.max_len))
            .collect()
    }
}

fn classify_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "label": { "type": "string" },
            "score": { "type": "number" },
        },
        "required": ["label", "score"],
        "additionalProperties": false,
    })
}

fn tokens_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "entities": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string" },
                        "text": { "type": "string" },
                        "score": { "type": "number" },
                    },
                    "required": ["label", "text", "score"],
                    "additionalProperties": false,
                },
            },
        },
        "required": ["entities"],
        "additionalProperties": false,
    })
}
