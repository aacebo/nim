pub mod captions;
pub mod classify;
pub mod transcribe;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models;

    pub fn classify() -> models::ModelRef {
        models::refer("wsntxxn/cnn8rnn-audioset-sed").expect("built-in model ids are valid")
    }

    pub fn transcribe() -> models::ModelRef {
        models::refer("openai/whisper-small").expect("built-in model ids are valid")
    }

    pub fn captions() -> models::ModelRef {
        models::refer("wsntxxn/cnn14rnn-tempgru-audiocaps-captioning").expect("built-in model ids are valid")
    }
}
