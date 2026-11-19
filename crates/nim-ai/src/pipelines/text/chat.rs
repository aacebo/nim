use crate::models::{ChatOptions, ModelRef, load};
use crate::tasks;
use crate::types::TextOutput;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,

    #[serde(default)]
    pub system: String,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::chat(),
            system: String::new(),
            beams: None,
            max_len: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> nim_error::Result<Vec<TextOutput>> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<TextOutput>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let model = load(&options.model)?;
    let capable = model.as_chat().ok_or_else(|| model.cannot("chat"))?;
    let options = ChatOptions {
        prompt: (!options.system.is_empty()).then_some(&options.system),
        beams: options.beams,
        max_len: options.max_len,
    };

    tasks::chat(capable, &model.context(), input, options)
}
