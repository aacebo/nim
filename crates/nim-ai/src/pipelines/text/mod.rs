pub mod classify;
pub mod complete;
pub mod embeddings;
pub mod split;
pub mod summarize;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models;

    pub fn embed() -> models::ModelRef {
        models::refer("sentence-transformers/all-MiniLM-L6-v2").expect("built-in model ids are valid")
    }

    pub fn keywords() -> models::ModelRef {
        models::refer("sentence-transformers/all-MiniLM-L6-v2").expect("built-in model ids are valid")
    }

    pub fn classify() -> models::ModelRef {
        models::refer("distilbert-base-uncased-finetuned-sst-2-english").expect("built-in model ids are valid")
    }

    pub fn zero_shot() -> models::ModelRef {
        models::refer("valhalla/distilbart-mnli-12-3").expect("built-in model ids are valid")
    }

    pub fn token_classify() -> models::ModelRef {
        models::refer("dbmdz/bert-base-cased-finetuned-conll03-english").expect("built-in model ids are valid")
    }

    pub fn summarize() -> models::ModelRef {
        models::refer(models::qwen2::QUANTIZED_MODEL).expect("built-in model ids are valid")
    }

    pub fn split() -> models::ModelRef {
        models::refer("mirth/chonky_distilbert_base_uncased_1").expect("built-in model ids are valid")
    }

    pub fn complete() -> models::ModelRef {
        models::refer(models::qwen2::QUANTIZED_MODEL).expect("built-in model ids are valid")
    }
}
