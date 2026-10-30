use hf_hub::{Cache, Repo};

use crate::Error;
use crate::models::ModelId;
use crate::resources::{DataSource, Resource, cache};

use super::HuggingFaceRepository;

pub struct HuggingFace;

impl HuggingFace {
    pub fn cached_resource(&self, id: &ModelId) -> Result<Option<Resource>, Error> {
        let cache = cache::dir().map(Cache::new).unwrap_or_default();
        let repo = cache.repo(Repo::model(id.to_string()));

        repo.get("").map(Resource::from_path).transpose()
    }
}

impl DataSource for HuggingFace {
    type Repository = HuggingFaceRepository;

    fn load(&self, id: &ModelId) -> Result<Self::Repository, Error> {
        HuggingFaceRepository::new(id)
    }
}
