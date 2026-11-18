use crate::{
    models::Context,
    types::{Annotation, Span},
};

pub trait Classify: Send + Sync {
    fn classify(&self, ctx: &Context, input: &[&[u8]], options: ClassifyOptions<'_>) -> nim_error::Result<Vec<Vec<Annotation>>>;
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
