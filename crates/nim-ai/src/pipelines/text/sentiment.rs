use crate::models::{ModelArgs, load};
use crate::types::{Annotation, Offset};
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

pub fn run(input: &[&str]) -> Result<Vec<Annotation>, Error> {
    run_with(input, Default::default())
}

pub fn run_with(input: &[&str], options: Options) -> Result<Vec<Annotation>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::classify())?, &api_key)?;
    let capable = model
        .as_sequence_classify()
        .ok_or_else(|| model.cannot("sequence-classify"))?;
    let out = tasks::sentiment(capable, &model.context(), input)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, sentiment) in out.into_iter().enumerate() {
        if sentiment.score < options.min_score {
            continue;
        }

        let source = input[index];
        let label = sentiment.polarity.as_str();

        annotations.push(Annotation {
            name: String::from("sentiment"),
            label: label.to_string(),
            text: label.to_string(),
            score: sentiment.score,
            spans: vec![Offset::new(0, source.chars().count() as u32)],
        });
    }

    Ok(annotations)
}
