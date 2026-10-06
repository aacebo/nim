use std::path::{Path, PathBuf};

use hf_hub::api::sync::{ApiBuilder, ApiRepo};

use crate::Error;
use crate::models::ModelId;
use crate::resources::{Asset, AssetData, Repository, cache};

pub struct HuggingFace {
    repo: Box<ApiRepo>,
}

impl HuggingFace {
    pub fn new(id: &ModelId) -> Result<Self, Error> {
        let mut builder = ApiBuilder::new().with_token(std::env::var("HF_TOKEN").ok());

        if let Some(dir) = cache::dir() {
            builder = builder.with_cache_dir(dir);
        }

        let api = builder.build()?;

        Ok(Self {
            repo: Box::new(api.model(id.to_string())),
        })
    }
}

impl Repository for HuggingFace {
    fn exists(&self, path: &Path) -> bool {
        self.resolve(path).is_ok()
    }

    fn get(&self, path: &Path) -> Result<Asset, Error> {
        Ok(Asset::file(self.resolve(path)?))
    }

    fn read(&self, path: &Path) -> Result<AssetData, Error> {
        let path = self.resolve(path)?;
        Ok(AssetData::File(std::fs::read(path)?))
    }

    fn resolve(&self, path: &Path) -> Result<PathBuf, Error> {
        let url = url::Url::from_file_path(path).map_err(|_| Error::custom("url::parse", format!("{path:?} is not utf-8")))?;
        Ok(self.repo.get(url.as_str())?)
    }
}
