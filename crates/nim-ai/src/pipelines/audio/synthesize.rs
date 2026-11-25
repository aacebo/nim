use crate::{
    SynthesizeOptions,
    models::{ModelRef, load},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
    pub synthesize: SynthesizeOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::synthesize(),
            synthesize: Default::default(),
        }
    }
}

pub async fn run(input: &[&str]) -> nim_error::Result<Vec<u8>> {
    run_with(input, Default::default()).await
}

pub async fn run_with(input: &[&str], options: Options) -> nim_error::Result<Vec<u8>> {
    options.synthesize.validate()?;
    let model = load(&options.model).await?;
    let capable = model.as_synthesize().ok_or_else(|| model.cannot("synthesize"))?;
    capable.synthesize(&model.context(), input, options.synthesize).await
}
