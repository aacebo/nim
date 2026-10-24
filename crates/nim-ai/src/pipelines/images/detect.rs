use crate::Error;
use crate::models::{ClassifyMode, ClassifyOptions, ModelArgs, load};
use crate::types::Annotation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
    pub min_score: f64,
    pub limit: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: Default::default(),
            min_score: 0.2,
            limit: 100,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Vec<Annotation>>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Vec<Annotation>>, Error> {
    let classify_options = ClassifyOptions {
        mode: ClassifyMode::Regions,
        min_score: options.min_score,
        limit: Some(options.limit),
        ..Default::default()
    };
    classify_options.validate()?;

    if input.is_empty() {
        return Ok(Vec::new());
    }

    if options.limit == 0 {
        return Ok(vec![Vec::new(); input.len()]);
    }

    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::detect())?, &api_key)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    crate::tasks::classify(capable, &model.context(), input, classify_options.clone())
}
