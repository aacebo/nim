use crate::Error;
use crate::models::{ModelArgs, load};
use crate::types::Annotation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
    pub limit: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: Default::default(),
            limit: 5,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[Vec<u8>]) -> Result<Vec<Vec<Annotation>>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[Vec<u8>], options: Options) -> Result<Vec<Vec<Annotation>>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::classify())?, &api_key)?;
    let capable = model.as_audio_classify().ok_or_else(|| model.cannot("audio-classify"))?;
    let mut out = vec![];

    for audio in input {
        let labels = capable.classify_audio(&model.context(), audio, options.limit)?;

        out.push(
            labels
                .iter()
                .map(|label| Annotation {
                    name: label.label.clone(),
                    label: label.label.clone(),
                    text: label.label.clone(),
                    score: label.score,
                    spans: Default::default(),
                })
                .collect(),
        );
    }

    Ok(out)
}
