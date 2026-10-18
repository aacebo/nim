pub mod captions;
pub mod classify;
pub mod transcribe;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models::{ModelId, ModelRef};

    pub fn hub(repo: &str) -> ModelRef {
        ModelRef::hub(repo.parse::<ModelId>().expect("built-in model ids are valid"))
    }

    pub fn classify() -> ModelRef {
        hub("wsntxxn/cnn8rnn-audioset-sed")
    }

    pub fn transcribe() -> ModelRef {
        hub("openai/whisper-small")
    }

    pub fn captions() -> ModelRef {
        hub("wsntxxn/cnn14rnn-tempgru-audiocaps-captioning")
    }
}
