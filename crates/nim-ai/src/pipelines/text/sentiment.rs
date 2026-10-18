use crate::models::{ModelArgs, load};
use crate::types::{Annotation, Offset};
use crate::{Error, tasks};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub text: Vec<String>,
    pub model: ModelArgs,
    pub min_score: f64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            text: Default::default(),
            model: Default::default(),
            min_score: 0.8,
        }
    }
}

pub fn sentiment(args: Args) -> Result<Vec<Annotation>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::classify())?, &api_key)?;
    let capable = model
        .as_sequence_classify()
        .ok_or_else(|| model.cannot("sequence-classify"))?;
    let out = tasks::sentiment(
        capable,
        &model.context(),
        &args.text.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, sentiment) in out.into_iter().enumerate() {
        if sentiment.score < args.min_score {
            continue;
        }

        let source = args.text.get(index).map(String::as_str).unwrap_or_default();
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
