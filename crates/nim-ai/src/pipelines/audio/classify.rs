use crate::models::{ClassifyMedia, ClassifyOptions, ModelArgs, load};
use crate::types::Annotation;
use crate::{Error, tasks};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
    pub min_score: f64,

    #[serde(default)]
    pub limit: Option<usize>,

    #[serde(default)]
    pub labels: Option<Vec<String>>,

    #[serde(default)]
    pub multi_label: bool,

    #[serde(default)]
    pub hypothesis_template: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: Default::default(),
            min_score: 0.0,
            limit: Some(5),
            labels: None,
            multi_label: false,
            hypothesis_template: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Vec<Annotation>>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Vec<Annotation>>, Error> {
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

    let api_key = options.model.api_key.clone();
    let default = super::defaults::classify();
    let model = load(&options.model.resolve(default)?, &api_key)?;
    classify_options.mode = model.classify_mode(ClassifyMedia::Audio)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::classify(capable, &model.context(), input, classify_options)?;
    Ok(out)
}
