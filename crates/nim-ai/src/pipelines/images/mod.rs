pub mod captions;
pub mod classify;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models::{ModelId, ModelRef};

    pub fn hub(repo: &str) -> ModelRef {
        ModelRef::hub(repo.parse::<ModelId>().expect("built-in model ids are valid"))
    }

    pub fn classify() -> ModelRef {
        hub("google/mobilenet_v2_1.0_224")
    }

    pub fn captions() -> ModelRef {
        hub("microsoft/git-base")
    }
}
