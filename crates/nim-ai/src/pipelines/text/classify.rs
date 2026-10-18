use crate::models::{Model, ModelArgs, load};
use crate::pipelines::TOP_N;
use crate::types::{Annotation, Entity, Offset};
use crate::{Error, tasks};

type TokenArgs = (Vec<String>, f64, std::sync::Arc<Model>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub text: Vec<String>,
    pub model: ModelArgs,
    pub min_score: f64,
}

impl Args {
    pub fn into_token_args(self) -> Result<TokenArgs, Error> {
        let api_key = self.model.api_key.clone();
        let model = load(&self.model.resolve(super::defaults::token_classify())?, &api_key)?;
        Ok((self.text, self.min_score, model))
    }
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

pub fn keywords(args: Args) -> Result<Vec<Annotation>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::keywords())?, &api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::keywords(
        capable,
        &model.context(),
        &args.text.iter().map(String::as_str).collect::<Vec<_>>(),
        TOP_N,
    )?;
    let min_score = args.min_score as f32;
    let mut annotations: Vec<Annotation> = Vec::new();

    for keywords in out {
        for keyword in keywords.into_iter().filter(|k| k.score >= min_score) {
            annotations.push(Annotation {
                name: String::from("keyword"),
                label: keyword.text.clone(),
                text: keyword.text,
                score: keyword.score as f64,
                spans: keyword.offsets.iter().map(|o| Offset::new(o.begin, o.end)).collect(),
            });
        }
    }

    Ok(annotations)
}

pub fn entities(args: Args) -> Result<Vec<Annotation>, Error> {
    let (text, min_score, model) = args.into_token_args()?;
    let capable = model.as_entity_classify().ok_or_else(|| model.cannot("entity-classify"))?;
    let out = tasks::entities(
        capable,
        &model.context(),
        &text.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities.into_iter().filter(|e| e.score >= min_score) {
            annotations.push(Annotation {
                name: "entity".to_string(),
                text: entity.word,
                score: entity.score,
                spans: vec![Offset::new(entity.offset.begin, entity.offset.end)],
                label: match entity.label.as_str() {
                    "ORG" => "organization".to_string(),
                    "PER" => "person".to_string(),
                    "LOC" => "location".to_string(),
                    other => other.to_lowercase(),
                },
            });
        }
    }

    Ok(annotations)
}

pub fn pii(args: Args) -> Result<Vec<Vec<Entity>>, Error> {
    let (text, min_score, model) = args.into_token_args()?;
    let capable = model.as_entity_classify().ok_or_else(|| model.cannot("entity-classify"))?;
    let mut out = tasks::entities(
        capable,
        &model.context(),
        &text.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;

    for entities in &mut out {
        entities.retain(|entity| entity.score >= min_score);
    }

    Ok(out)
}
