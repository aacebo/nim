use std::path::Path;

use hf_hub::api::Progress;
use hf_hub::api::sync::{ApiBuilder, ApiRepo};
use hf_hub::{Cache, CacheRepo, Repo};

use crate::clients::download::Download;
use crate::models::ModelId;
use crate::resources::{Asset, AssetData, Repository, Resource, Uri, cache};

pub struct HuggingFaceRepository {
    repo: Box<ApiRepo>,
    cache: CacheRepo,
    id: String,
}

impl HuggingFaceRepository {
    pub fn new(id: &ModelId) -> nim_error::Result<Self> {
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

    fn get(&self, path: &Path) -> nim_error::Result<Asset> {
        Ok(Asset::file(self.resolve(path)?.local_path()?))
    }

    fn read(&self, path: &Path) -> nim_error::Result<AssetData> {
        let path = self.resolve(path)?;
        Ok(AssetData::File(std::fs::read(path.local_path()?)?))
    }

    fn resolve(&self, path: &Path) -> nim_error::Result<Resource> {
        if path.as_os_str().is_empty() {
            let cached = self.cache.get("").ok_or_else(|| {
                nim_error::Diagnostic::from(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} has no cached repository", self.id),
                ))
            })?;

            return Resource::from_path(cached);
        }

        let path = path
            .to_str()
            .ok_or_else(|| nim_error::Diagnostic::message(format!("{path:?} is not utf-8")))?;
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
