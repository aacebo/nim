use crate::Error;
use crate::models::{ModelArgs, load};
use crate::types::Annotation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub images: Vec<Vec<u8>>,
    pub model: ModelArgs,
    pub limit: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            images: Default::default(),
            model: Default::default(),
            limit: 5,
        }
    }
}

pub fn run(args: Args) -> Result<Vec<Vec<Annotation>>, Error> {
    let api_key = args.model.api_key.clone();
    let model = load(&args.model.resolve(super::defaults::classify())?, &api_key)?;
    let capable = model.as_image_classify().ok_or_else(|| model.cannot("image-classify"))?;
    let mut out = vec![];

    for image in &args.images {
        let labels = capable.classify_image(&model.context(), image, args.limit)?;

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
