use crate::models::{GenerateOptions, Generation, ModelRef, load};
use crate::tasks;

const SYSTEM: &str = "You are NIM, a helpful assistant. Answer the user's message directly.";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,

    #[serde(default = "system")]
    pub system: String,

    #[serde(default)]
    pub beams: Option<usize>,

    #[serde(default)]
    pub max_len: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::generate(),
            system: system(),
            beams: None,
            max_len: None,
        }
    }
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()))]
pub fn run(input: &[&[u8]]) -> nim_error::Result<Vec<Generation>> {
    run_with(input, Default::default())
}

#[tracing::instrument(skip_all, fields(batch_size = input.len()), err)]
pub fn run_with(input: &[&[u8]], options: Options) -> nim_error::Result<Vec<Generation>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let model = load(&options.model)?;
    let capable = model.as_generate().ok_or_else(|| model.cannot("chat"))?;
    let options = GenerateOptions {
        prompt: Some(&options.system),
        beams: options.beams,
        max_len: options.max_len,
        ..GenerateOptions::default()
    };

    tasks::generate(capable, &model.context(), input, options)
}

fn system() -> String {
    SYSTEM.to_string()
}
