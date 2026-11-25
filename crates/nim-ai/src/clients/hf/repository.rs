use std::path::Path;

use hf_hub::api::tokio::{ApiBuilder, ApiRepo, Progress};
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
    fn exists<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, bool> {
        Box::pin(async move { self.resolve(path).await.is_ok() })
    }

    fn get<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<Asset>> {
        Box::pin(async move { Ok(Asset::file(self.resolve(path).await?.local_path()?)) })
    }

    fn read<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<AssetData>> {
        Box::pin(async move {
            let path = self.resolve(path).await?;
            Ok(AssetData::File(tokio::fs::read(path.local_path()?).await?))
        })
    }

    fn resolve<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<Resource>> {
        Box::pin(async move {
            if path.as_os_str().is_empty() {
                let cached = self.cache.get("").ok_or_else(|| {
                    nim_error::Diagnostic::from(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("{} has no cached repository", self.id),
                    ))
                })?;

                return Resource::from_path(cached).await;
            }

            let path = path
                .to_str()
                .ok_or_else(|| nim_error::Diagnostic::message(format!("{path:?} is not utf-8")))?;
            let uri = Uri::parse(&self.repo.url(path))?;
            let progress = Download::new(&self.id, path);

            if let Some(cached) = self.cache.get(path) {
                progress.cached();
                return Ok(Resource::new(uri).resolved(cached.clone(), tokio::fs::metadata(cached).await?.len()));
            }

            progress.start();

            let progress = DownloadProgress(std::sync::Arc::new(std::sync::Mutex::new(progress)));

            match self.repo.download_with_progress(path, progress.clone()).await {
                Ok(cached) => {
                    if let Ok(progress) = progress.0.lock() {
                        progress.complete();
                    }

                    Ok(Resource::new(uri).resolved(cached.clone(), tokio::fs::metadata(cached).await?.len()))
                }
                Err(error) => {
                    if let Ok(progress) = progress.0.lock() {
                        progress.failed();
                    }

                    Err(error.into())
                }
            }
        })
    }
}

#[derive(Clone)]
struct DownloadProgress(std::sync::Arc<std::sync::Mutex<Download>>);

impl Progress for DownloadProgress {
    fn init(&mut self, size: usize, _filename: &str) -> impl Future<Output = ()> + Send {
        if let Ok(mut progress) = self.0.lock() {
            progress.reset(Some(size as u64));
        }

        std::future::ready(())
    }

    fn update(&mut self, size: usize) -> impl Future<Output = ()> + Send {
        if let Ok(mut progress) = self.0.lock() {
            progress.update(size);
        }

        std::future::ready(())
    }

    fn finish(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}
