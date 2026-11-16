use candle_core::Device;
use tokenizers::Tokenizer;

use crate::types::{Annotation, Span};

use super::Batch;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Generation {
    pub text: String,
    pub confidence: Option<f64>,
}

pub trait Generate: Send + Sync {
    fn generate(&self, ctx: &Context, input: &[&[u8]], options: GenerateOptions<'_>) -> nim_error::Result<Vec<Generation>>;
}

/// A sentence vector per input.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EmbedMode {
    #[default]
    Raw,
    Query,
    Document,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EmbedOptions {
    pub mode: EmbedMode,
    pub instruction: Option<String>,
    pub dimensions: Option<usize>,
}

impl EmbedOptions {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.instruction.is_some() && self.mode != EmbedMode::Query {
            return Err(nim_error::Diagnostic::message("embedding instructions require query mode"));
        }

        Ok(())
    }
}

pub trait Embed: Send + Sync {
    fn embed(&self, ctx: &Context, input: &[&[u8]]) -> nim_error::Result<Vec<Vec<f32>>>;

    fn embed_with(&self, ctx: &Context, input: &[&[u8]], options: &EmbedOptions) -> nim_error::Result<Vec<Vec<f32>>> {
        options.validate()?;

        if options != &EmbedOptions::default() {
            return Err(nim_error::Diagnostic::message("model does not support embedding options"));
        }

        self.embed(ctx, input)
    }
}

pub trait Classify: Send + Sync {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Annotation>>>;
}

/// Borrows what a model needs to turn text into tensors. A loaded model owns its weights; the
/// tokenizer and device are lent per call, so one set of weights can serve several capabilities.
///
/// The tokenizer is optional because a hosted model has none -- it is handed text and returns
/// text. Its capability impls never touch the context, and `tokenizer()` is the error a local
/// model would raise if its tokenizer were somehow missing.
pub struct Context<'a> {
    name: &'a str,
    device: &'a Device,
    tokenizer: Option<&'a Tokenizer>,
}

impl<'a> Context<'a> {
    pub fn new(tokenizer: Option<&'a Tokenizer>, device: &'a Device, name: &'a str) -> Self {
        Self { tokenizer, device, name }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn device(&self) -> &Device {
        self.device
    }

    pub fn tokenizer(&self) -> nim_error::Result<&Tokenizer> {
        self.tokenizer
            .ok_or_else(|| nim_error::Diagnostic::message(format!("{} has no tokenizer", self.name)))
    }

    pub fn encode(&self, text: &[&str]) -> nim_error::Result<Batch> {
        let encodings = self
            .tokenizer()?
            .encode_batch(text.to_vec(), true)
            .map_err(nim_error::Diagnostic::message)?;
        Batch::new(encodings, self.device)
    }

    pub fn encode_one(&self, text: &str) -> nim_error::Result<tokenizers::Encoding> {
        self.tokenizer()?.encode(text, true).map_err(nim_error::Diagnostic::message)
    }
}

/// A whole word: sub-word pieces already merged, carrying BYTE offsets into the source text.
#[derive(Debug, Clone)]
pub struct Word {
    pub label: String,
    pub score: f64,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ClassifyMode {
    #[default]
    Whole,
    Regions,
}

#[derive(Debug, Clone)]
pub struct ClassifyOptions<'a> {
    pub mode: ClassifyMode,
    pub min_score: f64,
    pub limit: Option<usize>,
    pub labels: Option<&'a [String]>,
    pub multi_label: bool,
    pub hypothesis_template: &'a str,
}

impl Default for ClassifyOptions<'_> {
    fn default() -> Self {
        Self {
            mode: ClassifyMode::Whole,
            min_score: 0.0,
            limit: None,
            labels: None,
            multi_label: false,
            hypothesis_template: "This example is about {}.",
        }
    }
}

impl ClassifyOptions<'_> {
    pub fn validate(&self) -> nim_error::Result<()> {
        if !self.min_score.is_finite() || !(0.0..=1.0).contains(&self.min_score) {
            return Err(nim_error::Diagnostic::message("min_score must be finite and between 0 and 1"));
        }

        if self.hypothesis_template.matches("{}").count() != 1 {
            return Err(nim_error::Diagnostic::message(
                "hypothesis_template must contain exactly one {} placeholder",
            ));
        }

        if let Some(labels) = self.labels {
            if labels.is_empty() || labels.iter().any(|label| label.trim().is_empty()) {
                return Err(nim_error::Diagnostic::message(
                    "labels must contain at least one nonblank label",
                ));
            }

            if labels
                .iter()
                .enumerate()
                .any(|(index, label)| labels[..index].contains(label))
            {
                return Err(nim_error::Diagnostic::message("labels must be unique"));
            }
        }

        Ok(())
    }

    pub fn apply(&self, mut annotations: Vec<Annotation>) -> nim_error::Result<Vec<Annotation>> {
        self.validate()?;

        if annotations.iter().any(|annotation| {
            !annotation.score.is_finite()
                || !(0.0..=1.0).contains(&annotation.score)
                || self.labels.is_some_and(|labels| !labels.contains(&annotation.label))
        }) {
            return Err(nim_error::Diagnostic::message(
                "classification returned an invalid label or score",
            ));
        }

        annotations.retain(|annotation| annotation.score >= self.min_score);

        if let Some(limit) = self.limit {
            annotations.sort_by(|left, right| right.score.total_cmp(&left.score));
            annotations.truncate(limit);
        }

        if self.mode == ClassifyMode::Regions
            && annotations.iter().all(|annotation| {
                !annotation.spans.is_empty() && annotation.spans.iter().all(|span| matches!(span, Span::Text(_)))
            })
        {
            annotations.sort_by_key(|annotation| annotation.spans[0].as_text().map(|offset| offset.begin));
        }

        Ok(annotations)
    }
}

#[derive(Debug, Default, Clone)]
pub struct GenerateOptions<'a> {
    pub prompt: Option<&'a str>,
    pub language: Option<&'a str>,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl GenerateOptions<'_> {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
