use std::path::{Path, PathBuf};

use hf_hub::api::Progress;
use hf_hub::api::sync::{ApiBuilder, ApiRepo};
use hf_hub::{Cache, CacheRepo, Repo};

use crate::Error;
use crate::clients::download::Download;
use crate::models::ModelId;
use crate::resources::{Asset, AssetData, Repository, cache};

pub struct HuggingFace {
    repo: Box<ApiRepo>,
    cache: CacheRepo,
    id: String,
}

impl HuggingFace {
    pub fn new(id: &ModelId) -> Result<Self, Error> {
        let cache = cache::dir().map(Cache::new).unwrap_or_default();
        let api = ApiBuilder::from_cache(cache.clone())
            .with_token(std::env::var("HF_TOKEN").ok())
            .with_progress(false)
            .build()?;
        let id = id.to_string();

        Ok(Self {
            repo: Box::new(api.model(id.clone())),
            cache: cache.repo(Repo::model(id.clone())),
            id,
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
        let path = path
            .to_str()
            .ok_or_else(|| Error::custom("url::parse", format!("{path:?} is not utf-8")))?;
        let mut progress = Download::new(&self.id, path);

        if let Some(cached) = self.cache.get(path) {
            progress.cached();
            return Ok(cached);
        }

        progress.start();

        match self.repo.download_with_progress(path, &mut progress) {
            Ok(cached) => {
                progress.complete();
                Ok(cached)
            }
            Err(error) => {
                progress.failed();
                Err(error.into())
            }
        }
    }
}

impl Progress for &mut Download {
    fn init(&mut self, size: usize, _filename: &str) {
        self.reset(Some(size as u64));
    }

    fn update(&mut self, size: usize) {
        Download::update(self, size);
    }

    fn finish(&mut self) {}
}
