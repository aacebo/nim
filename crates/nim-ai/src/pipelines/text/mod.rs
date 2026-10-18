pub mod classify;
pub mod embeddings;
pub mod generate;
pub mod sentiment;
pub mod summarize;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models::{ModelId, ModelRef};

    pub fn hub(repo: &str) -> ModelRef {
        ModelRef::hub(repo.parse::<ModelId>().expect("built-in model ids are valid"))
    }

    pub fn embed() -> ModelRef {
        hub("sentence-transformers/all-MiniLM-L12-v2")
    }

    pub fn keywords() -> ModelRef {
        hub("sentence-transformers/all-MiniLM-L6-v2")
    }

    pub fn classify() -> ModelRef {
        hub("distilbert-base-uncased-finetuned-sst-2-english")
    }

    pub fn token_classify() -> ModelRef {
        hub("dbmdz/bert-large-cased-finetuned-conll03-english")
    }

    pub fn generate() -> ModelRef {
        hub("facebook/bart-large-cnn")
    }
}
