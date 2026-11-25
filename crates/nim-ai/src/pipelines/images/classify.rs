use crate::tasks;
use crate::types::Classification;
use crate::{
    ClassifyMedia, ClassifyOptions,
    models::{ModelRef, load},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
    pub min_score: f64,
    pub limit: Option<usize>,
    pub labels: Option<Vec<String>>,
    pub multi_label: bool,
    pub hypothesis_template: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::classify(),
            min_score: 0.0,
            limit: Some(5),
            labels: None,
            multi_label: false,
            hypothesis_template: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub async fn run(input: &[&[u8]]) -> nim_error::Result<Vec<Vec<Classification>>> {
    run_with(input, Default::default()).await
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub async fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<Vec<Classification>>> {
    let mut classify_options = ClassifyOptions {
        min_score: options.min_score,
        limit: options.limit,
        labels: options.labels.as_deref(),
        multi_label: options.multi_label,
        hypothesis_template: options.hypothesis_template.as_deref().unwrap_or("This example is about {}."),
        ..Default::default()
    };

    classify_options.validate()?;

    if input.is_empty() {
        return Ok(Vec::new());
    }

    let model = load(&options.model).await?;
    classify_options.mode = model.classify_mode(ClassifyMedia::Image)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::classify(capable, &model.context(), input, classify_options).await?;
    Ok(out)
}
