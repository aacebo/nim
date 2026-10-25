pub mod classify;
pub mod embeddings;
pub mod generate;
pub mod summarize;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::{model_id, models::ModelRef};

    pub fn embed() -> ModelRef {
        let id = model_id!("sentence-transformers/all-MiniLM-L12-v2").expect("built-in model ids are valid");
        ModelRef::hub(id)
    }

    pub fn keywords() -> ModelRef {
        let id = model_id!("sentence-transformers/all-MiniLM-L6-v2").expect("built-in model ids are valid");
        ModelRef::hub(id)
    }

    pub fn classify() -> ModelRef {
        let id = model_id!("distilbert-base-uncased-finetuned-sst-2-english").expect("built-in model ids are valid");
        ModelRef::hub(id)
    }

    pub fn zero_shot() -> ModelRef {
        let id = model_id!("facebook/bart-large-mnli").expect("built-in model ids are valid");
        ModelRef::hub(id)
    }

    pub fn token_classify() -> ModelRef {
        let id = model_id!("dbmdz/bert-large-cased-finetuned-conll03-english").expect("built-in model ids are valid");
        ModelRef::hub(id)
    }

    pub fn generate() -> ModelRef {
        let id = model_id!("facebook/bart-large-cnn").expect("built-in model ids are valid");
        ModelRef::hub(id)
    }
}
