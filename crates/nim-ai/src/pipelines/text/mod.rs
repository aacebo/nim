pub mod classify;
pub mod embeddings;
pub mod generate;
pub mod summarize;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models;

    pub fn embed() -> models::ModelRef {
        models::refer("sentence-transformers/all-MiniLM-L12-v2").expect("built-in model ids are valid")
    }

    pub fn keywords() -> models::ModelRef {
        models::refer("sentence-transformers/all-MiniLM-L6-v2").expect("built-in model ids are valid")
    }

    pub fn classify() -> models::ModelRef {
        models::refer("distilbert-base-uncased-finetuned-sst-2-english").expect("built-in model ids are valid")
    }

    pub fn zero_shot() -> models::ModelRef {
        models::refer("facebook/bart-large-mnli").expect("built-in model ids are valid")
    }

    pub fn token_classify() -> models::ModelRef {
        models::refer("dbmdz/bert-large-cased-finetuned-conll03-english").expect("built-in model ids are valid")
    }

    pub fn generate() -> models::ModelRef {
        models::refer("facebook/bart-large-cnn").expect("built-in model ids are valid")
    }
}
