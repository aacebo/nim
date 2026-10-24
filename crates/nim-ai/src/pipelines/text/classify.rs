use crate::models::{ClassifyMode, ClassifyOptions, ModelArgs, load};
use crate::pipelines::TOP_N;
use crate::types::Annotation;
use crate::{Error, tasks};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelArgs,
    pub min_score: f64,

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
            min_score: 0.8,
            labels: None,
            multi_label: false,
            hypothesis_template: None,
        }
    }
}

impl Options {
    fn classify_options(&self, mode: ClassifyMode) -> Result<ClassifyOptions<'_>, Error> {
        let options = ClassifyOptions {
            mode,
            labels: self.labels.as_deref(),
            multi_label: self.multi_label,
            hypothesis_template: self.hypothesis_template.as_deref().unwrap_or("This example is about {}."),
        };
        options.validate()?;

        Ok(options)
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&str]) -> Result<Vec<Annotation>, Error> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&str], options: Options) -> Result<Vec<Annotation>, Error> {
    let classify_options = options.classify_options(ClassifyMode::Text)?;
    let api_key = options.model.api_key.clone();
    let default = match &options.labels {
        Some(_) => super::defaults::zero_shot(),
        None => super::defaults::classify(),
    };
    let model = load(&options.model.clone().resolve(default)?, &api_key)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::classify(capable, &model.context(), input, &classify_options)?;
    let annotations: Vec<Annotation> = out
        .into_iter()
        .flatten()
        .filter(|annotation| annotation.score >= options.min_score)
        .collect();

    Ok(annotations)
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn keywords(input: &[&str]) -> Result<Vec<Annotation>, Error> {
    keywords_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn keywords_with(input: &[&str], options: Options) -> Result<Vec<Annotation>, Error> {
    let classify_options = options.classify_options(ClassifyMode::Spans)?;
    let api_key = options.model.api_key.clone();
    let default = match &options.labels {
        Some(_) => super::defaults::zero_shot(),
        None => super::defaults::keywords(),
    };
    let model = load(&options.model.clone().resolve(default)?, &api_key)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    if options.labels.is_some() {
        let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
        let out = tasks::classify(capable, &model.context(), input, &classify_options)?;

        for annotations_for_input in out {
            let retained = tasks::zero_shot::retain(annotations_for_input, &classify_options, options.min_score)?;
            annotations.extend(merge_keywords(retained));
        }

        return Ok(annotations);
    }

    let capable = model.as_embed().ok_or_else(|| model.cannot("embed"))?;
    let out = tasks::keywords(capable, &model.context(), input, TOP_N)?;

    for (text, keywords) in input.iter().zip(out) {
        let mut annotations_for_input: Vec<Annotation> = Vec::new();

        for keyword in keywords
            .into_iter()
            .filter(|keyword| keyword.score >= options.min_score as f32)
        {
            for span in keyword.offsets {
                annotations_for_input.push(Annotation {
                    name: String::from("keyword"),
                    text: text
                        .chars()
                        .skip(span.begin as usize)
                        .take((span.end - span.begin) as usize)
                        .collect(),
                    label: keyword.text.clone(),
                    score: keyword.score as f64,
                    spans: vec![span],
                });
            }
        }

        annotations.extend(merge_keywords(annotations_for_input));
    }

    Ok(annotations)
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn entities(input: &[&str]) -> Result<Vec<Annotation>, Error> {
    entities_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn entities_with(input: &[&str], options: Options) -> Result<Vec<Annotation>, Error> {
    let classify_options = options.classify_options(ClassifyMode::Spans)?;
    let api_key = options.model.api_key.clone();
    let default = match &options.labels {
        Some(_) => super::defaults::zero_shot(),
        None => super::defaults::token_classify(),
    };
    let model = load(&options.model.clone().resolve(default)?, &api_key)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::classify(capable, &model.context(), input, &classify_options)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for annotations_for_input in out {
        let annotations_for_input = if options.labels.is_some() {
            tasks::zero_shot::retain(annotations_for_input, &classify_options, options.min_score)?
        } else {
            annotations_for_input
        };

        for mut annotation in annotations_for_input
            .into_iter()
            .filter(|annotation| annotation.score >= options.min_score)
        {
            annotation.name = String::from("entity");

            if options.labels.is_none() {
                annotation.label = match annotation.label.as_str() {
                    "ORG" => String::from("organization"),
                    "PER" => String::from("person"),
                    "LOC" => String::from("location"),
                    other => other.to_lowercase(),
                };
            }

            annotations.push(annotation);
        }
    }

    Ok(annotations)
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn pii(input: &[&str]) -> Result<Vec<Vec<Annotation>>, Error> {
    pii_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn pii_with(input: &[&str], options: Options) -> Result<Vec<Vec<Annotation>>, Error> {
    let classify_options = options.classify_options(ClassifyMode::Spans)?;
    let api_key = options.model.api_key.clone();
    let default = match &options.labels {
        Some(_) => super::defaults::zero_shot(),
        None => super::defaults::token_classify(),
    };
    let model = load(&options.model.clone().resolve(default)?, &api_key)?;
    let capable = model.as_classify().ok_or_else(|| model.cannot("classify"))?;
    let out = tasks::classify(capable, &model.context(), input, &classify_options)?;
    let mut annotations: Vec<Vec<Annotation>> = Vec::new();

    for annotations_for_input in out {
        let mut annotations_for_input = if options.labels.is_some() {
            tasks::zero_shot::retain(annotations_for_input, &classify_options, options.min_score)?
        } else {
            annotations_for_input
        };
        annotations_for_input.retain(|annotation| annotation.score >= options.min_score);

        for annotation in &mut annotations_for_input {
            annotation.name = String::from("pii");
        }

        annotations.push(annotations_for_input);
    }

    Ok(annotations)
}

fn merge_keywords(annotations: Vec<Annotation>) -> Vec<Annotation> {
    let mut keywords: Vec<Annotation> = Vec::new();

    for mut annotation in annotations {
        if let Some(keyword) = keywords
            .iter_mut()
            .find(|keyword| keyword.text == annotation.text && keyword.label == annotation.label)
        {
            keyword.score = keyword.score.max(annotation.score);
            keyword.spans.extend(annotation.spans);
        } else {
            annotation.name = String::from("keyword");
            keywords.push(annotation);
        }
    }

    keywords.sort_by(|left, right| right.score.total_cmp(&left.score));
    keywords.truncate(TOP_N);
    keywords
}
