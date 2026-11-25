use crate::types::{TextOutput, Word};
use crate::{
    SplitOptions,
    models::{ModelRef, load},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
    pub split: SplitOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::split(),
            split: Default::default(),
        }
    }
}

pub async fn run(input: &str) -> nim_error::Result<Vec<TextOutput>> {
    run_with(input, Default::default()).await
}

pub async fn run_with(input: &str, options: Options) -> nim_error::Result<Vec<TextOutput>> {
    let model = load(&options.model).await?;
    let capable = model.as_split().ok_or_else(|| model.cannot("split"))?;
    capable.split(&model.context(), input, options.split).await
}

pub async fn run_delimited(input: &str) -> nim_error::Result<Vec<Word>> {
    run_delimited_with(input, Default::default()).await
}

pub async fn run_delimited_with(input: &str, options: Options) -> nim_error::Result<Vec<Word>> {
    let model = load(&options.model).await?;
    let capable = model.as_split().ok_or_else(|| model.cannot("split"))?;
    capable.split_delimited(&model.context(), input, options.split).await
}
