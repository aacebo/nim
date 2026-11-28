use nim_chat::{Chat, ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatUserMessage, Media};

use crate::tasks::{anchor, zero_shot};
use crate::types::TextOutput;
use crate::types::{Classification, Offset, Span};

use super::RemoteModel;
use crate::{Classify, ClassifyMode, ClassifyOptions, Complete, Context, Embed, Summarize, SummarizeOptions};

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
    fn embed<'a>(
        &'a self,
        _ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            let text = input
                .iter()
                .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
                .collect::<Result<Vec<_>, _>>()?;

            if text.is_empty() {
                return Ok(Vec::new());
            }

            self.client().embeddings(&text).await
        })
    }
}

impl Classify for RemoteModel {
    fn classify<'a>(
        &'a self,
        _ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>> {
        Box::pin(async move {
            let text = input
                .iter()
                .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
                .collect::<Result<Vec<_>, _>>()?;
            options.validate()?;

            let out = match (options.mode, options.labels) {
                (ClassifyMode::Whole, Some(_)) => self.custom_text(&text, options.clone()).await,
                (ClassifyMode::Whole, None) => self.text(&text).await,
                (ClassifyMode::Regions, Some(_)) => self.custom_spans(&text, options.clone()).await,
                (ClassifyMode::Regions, None) => {
                    futures_util::future::try_join_all(text.iter().map(|text| self.spans(text))).await
                }
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
        })
    }
}

impl RemoteModel {
    async fn text(&self, text: &[&str]) -> nim_error::Result<Vec<Vec<Classification>>> {
        futures_util::future::try_join_all(text.iter().map(|text| async {
            let out: Classified = self.client().json(CLASSIFY, text, classify_schema()).await?;

            Ok(vec![Classification {
                name: String::from("classification"),

                label: out.label.to_lowercase(),
                score: out.score,
                spans: vec![Span::Offset(Offset::new(0, text.chars().count() as u32))],
            }])
        }))
        .await
    }

    async fn custom_text(&self, text: &[&str], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Classification>>> {
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

        futures_util::future::try_join_all(text.iter().map(|text| async {
            let out: Scores = self.client().json(&prompt, text, schema.clone()).await?;

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
        }))
        .await
    }

    async fn custom_spans(&self, text: &[&str], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Classification>>> {
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

        futures_util::future::try_join_all(text.iter().map(|text| async {
            let found: Found = self.client().json(&prompt, text, schema.clone()).await?;
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
        }))
        .await
    }

    async fn spans(&self, text: &str) -> nim_error::Result<Vec<Classification>> {
        let found: Found = self.client().json(TOKENS, text, tokens_schema()).await?;

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

/// Completion is plain, not schema-constrained: the output is prose, and
/// wrapping it in a JSON envelope buys nothing.
impl Complete for RemoteModel {
    fn complete<'a>(
        &'a self,
        _ctx: &'a Context<'_>,
        chat: Chat,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<ChatAgentMessage>> {
        Box::pin(async move {
            chat.validate()?;

            if chat.beams.is_some() {
                return Err(nim_error::Diagnostic::message("hosted completion does not support beams"));
            }

            let max_tokens = chat.max_tokens;
            let tools = chat
                .tools
                .into_iter()
                .map(|tool| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.params,
                        },
                    })
                })
                .collect();
            let mut messages = chat
                .documents
                .into_iter()
                .map(|document| {
                    Ok(serde_json::json!({
                        "role": "system",
                        "content": serde_json::to_string(&document)?,
                    }))
                })
                .collect::<nim_error::Result<Vec<_>>>()?;

            for message in chat.messages {
                if let Some(message) = render_message(message)? {
                    messages.push(message);
                }
            }

            self.client().complete(messages, tools, max_tokens).await
        })
    }
}

fn render_message(message: ChatMessage) -> nim_error::Result<Option<serde_json::Value>> {
    match message {
        ChatMessage::System(message) => {
            Ok(render_content(message.content)?.map(|content| serde_json::json!({ "role": "system", "content": content })))
        }
        ChatMessage::User(message) => {
            Ok(render_content(message.content)?.map(|content| serde_json::json!({ "role": "user", "content": content })))
        }
        ChatMessage::Agent(message) => {
            let mut thinking = Vec::new();
            let content = match message.content {
                Some(ChatMessageContent::Parts(parts)) => {
                    let mut content = Vec::new();

                    for part in parts {
                        match part {
                            ChatMessageContentPart::Thinking { thinking: value } => thinking.push(value),
                            value => content.push(value),
                        }
                    }

                    (!content.is_empty()).then_some(ChatMessageContent::Parts(content))
                }
                content => content,
            };
            let content = content.map(render_content).transpose()?.flatten();
            let tool_calls = message
                .tool_calls
                .into_iter()
                .map(|call| {
                    Ok(serde_json::json!({
                        "id": call.id,
                        "type": "function",
                        "function": {
                            "name": call.name,
                            "arguments": serde_json::to_string(&call.args)?,
                        },
                    }))
                })
                .collect::<nim_error::Result<Vec<_>>>()?;
            let mut value = serde_json::json!({
                "role": "assistant",
                "content": content,
                "tool_calls": tool_calls,
            });

            if !thinking.is_empty() {
                value["reasoning_content"] = serde_json::json!(thinking.join("\n"));
            }

            Ok(Some(value))
        }
        ChatMessage::Tool(message) => {
            let content = message.content.map(render_content).transpose()?.flatten();

            Ok(Some(serde_json::json!({
                "role": "tool",
                "tool_call_id": message.tool_call_id,
                "content": content,
            })))
        }
        _ => Err(nim_error::Diagnostic::message("unsupported message role")),
    }
}

fn render_content(content: ChatMessageContent) -> nim_error::Result<Option<serde_json::Value>> {
    match content {
        ChatMessageContent::String(text) => Ok(Some(serde_json::json!(text))),
        ChatMessageContent::Parts(parts) => {
            let mut content = Vec::new();

            for part in parts {
                if let Some(part) = render_content_part(part)? {
                    content.push(part);
                }
            }

            Ok((!content.is_empty()).then_some(serde_json::json!(content)))
        }
        _ => Err(nim_error::Diagnostic::message("unsupported message content")),
    }
}

fn render_content_part(part: ChatMessageContentPart) -> nim_error::Result<Option<serde_json::Value>> {
    let text = |text| Ok(Some(serde_json::json!({ "type": "text", "text": text })));

    match part {
        ChatMessageContentPart::Text { text: value } => text(value),
        ChatMessageContentPart::Json { json } => text(serde_json::to_string(&json)?),
        ChatMessageContentPart::Url { url } => text(url.to_string()),
        ChatMessageContentPart::Image { media } => render_media("image", media),
        ChatMessageContentPart::Video { media } => render_media("video", media),
        ChatMessageContentPart::Audio { media } => render_media("audio", media),
        _ => Ok(None),
    }
}

fn render_media(kind: &str, media: Media) -> nim_error::Result<Option<serde_json::Value>> {
    let (name, value, size, mime_type, summary) = match media {
        Media::Url {
            name,
            url,
            size,
            mime_type,
            summary,
        } => (name, serde_json::to_value(url)?, size, mime_type, summary),
        Media::Path {
            name,
            path,
            size,
            mime_type,
            summary,
        } => (name, serde_json::to_value(path)?, size, mime_type, summary),
        Media::Data { .. } | _ => return Ok(None),
    };
    let mut output = serde_json::Map::new();

    output.insert("type".into(), serde_json::json!(kind));
    output.insert("name".into(), serde_json::to_value(name)?);
    output.insert("size".into(), serde_json::to_value(size)?);
    output.insert("mime_type".into(), serde_json::to_value(mime_type)?);
    output.insert(kind.into(), value);
    output.insert("summary".into(), serde_json::to_value(summary)?);

    Ok(Some(serde_json::Value::Object(output)))
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
    fn summarize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: SummarizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        Box::pin(async move {
            options.validate()?;

            let mut output = Vec::with_capacity(input.len());

            for bytes in input {
                let text = std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from)?;
                let prompt = concat!(
                    "Summarize the text concisely and accurately, using only information present in the text. ",
                    "Return only the summary.",
                );
                let chat = Chat {
                    messages: vec![
                        ChatUserMessage {
                            name: None,
                            content: ChatMessageContent::String(format!("{prompt}\n\n{text}")),
                            extra: Default::default(),
                        }
                        .into(),
                    ],
                    beams: options.beams,
                    max_tokens: options.max_tokens,
                    ..Default::default()
                };

                let response = self.complete(ctx, chat).await?;

                output.push(TextOutput {
                    text: crate::tasks::response_text(response.content),
                    confidence: None,
                });
            }

            Ok(output)
        })
    }
}
