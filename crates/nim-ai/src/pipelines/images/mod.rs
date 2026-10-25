pub mod captions;
pub mod classify;

/// Default models, one per capability. A model that cannot serve the routine you called is an
/// error -- the empty cells of the capability matrix.
pub mod defaults {
    use crate::models;

    pub fn classify() -> models::ModelRef {
        models::refer("google/mobilenet_v2_1.0_224").expect("built-in model ids are valid")
    }

    pub fn captions() -> models::ModelRef {
        models::refer("microsoft/git-base").expect("built-in model ids are valid")
    }
}
