use crate::models::{ModelArgs, load};
use crate::pipelines::TOP_N;
use crate::types::{Annotation, Entity, Offset};
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
pub fn keywords(input: &[&str]) -> Result<Vec<Annotation>, Error> {
    keywords_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn keywords_with(input: &[&str], options: Options) -> Result<Vec<Annotation>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::keywords())?, &api_key)?;
    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::keywords(capable, &model.context(), input, TOP_N)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for keywords in out {
        for keyword in keywords.into_iter().filter(|k| k.score >= options.min_score as f32) {
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

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn entities(input: &[&str]) -> Result<Vec<Annotation>, Error> {
    entities_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn entities_with(input: &[&str], options: Options) -> Result<Vec<Annotation>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::keywords())?, &api_key)?;
    let capable = model.as_entity_classify().ok_or_else(|| model.cannot("entity-classify"))?;
    let out = tasks::entities(capable, &model.context(), input)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for entities in out {
        for entity in entities.into_iter().filter(|e| e.score >= options.min_score) {
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

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn pii(input: &[&str]) -> Result<Vec<Vec<Entity>>, Error> {
    pii_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn pii_with(input: &[&str], options: Options) -> Result<Vec<Vec<Entity>>, Error> {
    let api_key = options.model.api_key.clone();
    let model = load(&options.model.resolve(super::defaults::keywords())?, &api_key)?;
    let capable = model.as_entity_classify().ok_or_else(|| model.cannot("entity-classify"))?;
    let mut out = tasks::entities(capable, &model.context(), input)?;

    for entities in &mut out {
        entities.retain(|entity| entity.score >= options.min_score);
    }

    Ok(out)
}
