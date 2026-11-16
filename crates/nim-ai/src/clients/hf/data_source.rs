use crate::models::ModelId;
use crate::resources::DataSource;

use super::HuggingFaceRepository;

pub struct HuggingFace;

impl DataSource for HuggingFace {
    type Repository = HuggingFaceRepository;

    fn load(&self, id: &ModelId) -> nim_error::Result<Self::Repository> {
        HuggingFaceRepository::new(id)
    }
}
