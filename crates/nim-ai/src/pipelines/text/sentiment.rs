use crate::models::{ClassifyOptions, ModelArgs, load};
use crate::pipelines::common::decode_text;
use crate::types::{Annotation, Offset, Span};
use crate::{Error, tasks};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
    pub min_score: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: Default::default(),
            min_score: 0.8,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> Result<Vec<Annotation>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> Result<Vec<Annotation>, Error> {
    let text = decode_text(input)?;
    ClassifyOptions {
        min_score: options.min_score,
        ..Default::default()
    }
    .validate()?;
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::classify())?, &api_key)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::sentiment(capable, &model.context(), input)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, sentiment) in out.into_iter().enumerate() {
        if sentiment.score < options.min_score {
            continue;
        }

        let source = text[index];
        let label = sentiment.polarity.as_str();

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: label.to_string(),
            score: sentiment.score,
            spans: vec![Span::Text(Offset::new(0, source.chars().count() as u32))],
        });
    }

    Ok(annotations)
}
