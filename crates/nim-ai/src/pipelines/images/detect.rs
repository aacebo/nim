use crate::Error;
use crate::models::{Detection, ModelArgs, load};

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
            min_score: 0.5,
            limit: 100,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[Vec<u8>]) -> Result<Vec<Vec<Detection>>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[Vec<u8>], options: Options) -> Result<Vec<Vec<Detection>>, Error> {
    if !options.min_score.is_finite() || !(0.0..=1.0).contains(&options.min_score) {
        return Err(Error::custom("ai::config", "min_score must be finite and between 0 and 1"));
    }

    if input.is_empty() {
        return Ok(Vec::new());
    }

    if options.limit == 0 {
        return Ok(vec![Vec::new(); input.len()]);
    }

    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::detect())?, &api_key)?;
    let capable = model.as_image_detect().ok_or_else(|| model.cannot("image-detect"))?;
    input
        .iter()
        .map(|image| capable.detect_image(&model.context(), image, options.min_score, options.limit))
        .collect()
}
