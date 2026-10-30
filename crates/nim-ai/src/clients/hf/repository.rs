use std::path::Path;

use hf_hub::api::Progress;
use hf_hub::api::sync::{ApiBuilder, ApiRepo};
use hf_hub::{Cache, CacheRepo, Repo};

use crate::Error;
use crate::clients::download::Download;
use crate::models::ModelId;
use crate::resources::{Asset, AssetData, Repository, Resource, Uri, cache};

pub struct HuggingFaceRepository {
    repo: Box<ApiRepo>,
    cache: CacheRepo,
    id: String,
}

impl HuggingFaceRepository {
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

impl Repository for HuggingFaceRepository {
    fn exists(&self, path: &Path) -> bool {
        self.resolve(path).is_ok()
    }

    fn get(&self, path: &Path) -> Result<Asset, Error> {
        Ok(Asset::file(self.resolve(path)?.local_path()?))
    }

    fn read(&self, path: &Path) -> Result<AssetData, Error> {
        let path = self.resolve(path)?;
        Ok(AssetData::File(std::fs::read(path.local_path()?)?))
    }

    fn resolve(&self, path: &Path) -> Result<Resource, Error> {
        if path.as_os_str().is_empty() {
            let cached = self.cache.get("").ok_or_else(|| {
                Error::IO(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} has no cached repository", self.id),
                ))
            })?;

            return Resource::from_path(cached);
        }

        let path = path
            .to_str()
            .ok_or_else(|| Error::custom("url::parse", format!("{path:?} is not utf-8")))?;
        let uri = Uri::parse(&self.repo.url(path))?;
        let mut progress = Download::new(&self.id, path);

        if let Some(cached) = self.cache.get(path) {
            progress.cached();
            return Ok(Resource::new(uri).resolved(cached.clone(), std::fs::metadata(cached)?.len()));
        }

        progress.start();

        match self.repo.download_with_progress(path, &mut progress) {
            Ok(cached) => {
                progress.complete();
                Ok(Resource::new(uri).resolved(cached.clone(), std::fs::metadata(cached)?.len()))
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
