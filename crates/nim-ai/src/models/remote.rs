use crate::tasks::{anchor, zero_shot};
use crate::types::TextOutput;
use crate::types::{Classification, Offset, Span};

use super::RemoteModel;
use crate::{
    Chat, ChatAssistantMessage, ChatOptions, Classify, ClassifyMode, ClassifyOptions, Context, Embed, Summarize, SummarizeOptions,
};

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

/// A hosted model serves every capability: the classification ones through a JSON-schema-constrained
/// chat and summarization through plain completions, and embeddings through their own endpoint. This is the full remote row of the capability matrix.
impl Embed for RemoteModel {
    fn embed(&self, _ctx: &Context, input: &[&[u8]]) -> nim_error::Result<Vec<Vec<f32>>> {
        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        if text.is_empty() {
            return Ok(Vec::new());
        }

        self.client().embeddings(&text)
    }
}

impl Classify for RemoteModel {
    fn classify(
        &self,
        _ctx: &Context,
        input: &[&[u8]],
        options: ClassifyOptions<'_>,
    ) -> nim_error::Result<Vec<Vec<Classification>>> {
        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        options.validate()?;

        let out = match (options.mode, options.labels) {
            (ClassifyMode::Whole, Some(_)) => self.custom_text(&text, options.clone()),
            (ClassifyMode::Whole, None) => self.text(&text),
            (ClassifyMode::Regions, Some(_)) => self.custom_spans(&text, options.clone()),
            (ClassifyMode::Regions, None) => text.iter().map(|text| self.spans(text)).collect(),
        }?;

        out.into_iter()
            .map(|classifications| {
                let classifications = if options.mode == ClassifyMode::Regions && options.labels.is_some() {
                    zero_shot::retain(classifications, options.clone(), options.min_score)?
                } else {
                    classifications
                };
                options.apply(classifications)
            })
            .collect()
    }
}

impl RemoteModel {
    fn text(&self, text: &[&str]) -> nim_error::Result<Vec<Vec<Classification>>> {
        text.iter()
            .map(|text| {
                let out: Classified = self.client().json(CLASSIFY, text, classify_schema())?;

                Ok(vec![Classification {
                    name: String::from("classification"),

                    label: out.label.to_lowercase(),
                    score: out.score,
                    spans: vec![Span::Offset(Offset::new(0, text.chars().count() as u32))],
                }])
            })
            .collect()
    }

    fn custom_text(&self, text: &[&str], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Classification>>> {
        let candidates = options
            .labels
            .ok_or_else(|| nim_error::Diagnostic::message("custom labels are required"))?;
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
                    return Err(nim_error::Diagnostic::message(
                        "hosted classification returned invalid labels or scores",
                    ));
                }

                let mut labels = Vec::with_capacity(candidates.len());

                for label in candidates {
                    let matching: Vec<&Classified> = out.labels.iter().filter(|item| &item.label == label).collect();

                    if matching.len() != 1 {
                        return Err(nim_error::Diagnostic::message(
                            "hosted classification must return each label exactly once",
                        ));
                    }

                    labels.push(Classification {
                        name: String::from("classification"),

                        label: label.clone(),
                        score: matching[0].score,
                        spans: vec![Span::Offset(Offset::new(0, text.chars().count() as u32))],
                    });
                }

                if !options.multi_label && candidates.len() > 1 {
                    let total: f64 = labels.iter().map(|label| label.score).sum();

                    if total <= 0.0 {
                        return Err(nim_error::Diagnostic::message(
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

    fn custom_spans(&self, text: &[&str], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Classification>>> {
        let candidates = options
            .labels
            .ok_or_else(|| nim_error::Diagnostic::message("custom labels are required"))?;
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
                        return Err(nim_error::Diagnostic::message(
                            "hosted extraction returned an invalid label or score",
                        ));
                    }

                    for offset in anchor::all(text, &item.text) {
                        entities.push(Classification {
                            name: String::from("entity"),
                            spans: vec![Span::Offset(offset)],

                            label: item.label.clone(),
                            score: item.score,
                        });
                    }
                }

                Ok(entities)
            })
            .collect()
    }

    fn spans(&self, text: &str) -> nim_error::Result<Vec<Classification>> {
        let found: Found = self.client().json(TOKENS, text, tokens_schema())?;

        Ok(found
            .entities
            .into_iter()
            // A hosted API gives no offsets, so spans are re-anchored in the source. An entity
            // that is not verbatim cannot be anchored -- and violates the prompt -- so it is
            // dropped rather than given an invented span.
            .filter_map(|item| {
                Some(Classification {
                    name: String::from("entity"),
                    spans: vec![Span::Offset(anchor::find(text, &item.text)?)],

                    label: item.label.to_lowercase(),
                    score: item.score,
                })
            })
            .collect())
    }
}

/// Chat is a plain completion, not a schema-constrained one: the output is prose, and
/// wrapping it in a JSON envelope buys nothing.
impl Chat for RemoteModel {
    fn chat(&self, _ctx: &Context, options: ChatOptions) -> nim_error::Result<ChatAssistantMessage> {
        options.validate()?;

        if options.beams.is_some() {
            return Err(nim_error::Diagnostic::message("hosted chat does not support beams"));
        }

        if !options.documents.is_empty() || options.thinking || options.vision {
            if options.compat == crate::CompatMode::Error {
                return Err(nim_error::Diagnostic::message(
                    "remote chat does not support requested render options",
                ));
            }
        }

        let messages = options
            .messages
            .into_iter()
            .map(|message| message.into_openai(options.compat))
            .collect::<nim_error::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        let tools = options.tools.into_iter().map(crate::Tool::into_openai).collect();

        self.client().chat(messages, tools, options.max_len)
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

impl Summarize for RemoteModel {
    fn summarize(&self, ctx: &Context, input: &[&[u8]], options: SummarizeOptions) -> nim_error::Result<Vec<TextOutput>> {
        options.validate()?;
        input.iter().map(|bytes| {
            let text = std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from)?;
            let response = self.chat(ctx, ChatOptions {
                messages: vec![crate::ChatUserMessage { content: crate::ChatMessageContent::Text(format!("Summarize the text concisely and accurately, using only information present in the text. Return only the summary.\n\n{text}")) }.into()],
                beams: options.beams,
                max_len: options.max_len,
                ..Default::default()
            })?;
            Ok(crate::types::TextOutput {
                text: match response.content { Some(crate::ChatMessageContent::Text(text)) => text, _ => String::new() },
                confidence: None,
            })
        }).collect()
    }
}
