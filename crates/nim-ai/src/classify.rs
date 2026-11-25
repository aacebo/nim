use crate::{
    Context,
    types::{Classification, Span},
};

pub trait Classify: Send + Sync {
    fn classify<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: ClassifyOptions<'a>,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Vec<Classification>>>>;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ClassifyMode {
    #[default]
    Whole,
    Regions,
}

#[derive(Debug, Copy, Clone)]
pub enum ClassifyMedia {
    Text,
    #[cfg(feature = "vision")]
    Image,
    #[cfg(feature = "audio")]
    Audio,
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

    pub fn apply(&self, mut classifications: Vec<Classification>) -> nim_error::Result<Vec<Classification>> {
        self.validate()?;

        if classifications.iter().any(|classification| {
            !classification.score.is_finite()
                || !(0.0..=1.0).contains(&classification.score)
                || self.labels.is_some_and(|labels| !labels.contains(&classification.label))
        }) {
            return Err(nim_error::Diagnostic::message(
                "classification returned an invalid label or score",
            ));
        }

        classifications.retain(|classification| classification.score >= self.min_score);

        if let Some(limit) = self.limit {
            classifications.sort_by(|left, right| right.score.total_cmp(&left.score));
            classifications.truncate(limit);
        }

        if self.mode == ClassifyMode::Regions
            && classifications.iter().all(|classification| {
                !classification.spans.is_empty() && classification.spans.iter().all(|span| matches!(span, Span::Offset(_)))
            })
        {
            classifications.sort_by_key(|classification| classification.spans[0].as_offset().map(|offset| offset.begin));
        }

        Ok(classifications)
    }
}
