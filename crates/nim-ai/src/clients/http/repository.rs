use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    clients::download::Download,
    resources::{Asset, AssetData, Repository, Resource, Uri, cache},
};

pub struct HttpRepository {
    base: Uri,
}

impl HttpRepository {
    pub fn new(base: Uri) -> Self {
        Self { base }
    }
}

impl Repository for HttpRepository {
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
            let filename = path
                .to_str()
                .ok_or_else(|| nim_error::Diagnostic::message(format!("{path:?} is not utf-8")))?;
            let uri = self.base.join(filename)?;
            let key = format!("{:x}", Sha256::digest(uri.to_string().as_bytes()));
            let cached = cache::dir().unwrap_or_else(std::env::temp_dir).join("urls").join(key);
            let mut progress = Download::new(&self.base.to_string(), filename);

            if tokio::fs::try_exists(&cached).await? {
                progress.cached();
                return Ok(Resource::new(uri).resolved(cached.clone(), tokio::fs::metadata(cached).await?.len()));
            }

            progress.start();

            let result: nim_error::Result<PathBuf> = async {
                let url = uri.to_string();
                let mut response = reqwest::get(&url).await?.error_for_status()?;
                progress.reset(response.content_length());

                if let Some(dir) = cached.parent() {
                    tokio::fs::create_dir_all(dir).await?;
                }

                let mut bytes = Vec::new();

                while let Some(chunk) = response.chunk().await? {
                    progress.update(chunk.len());
                    bytes.extend_from_slice(&chunk);
                }

                tokio::fs::write(&cached, bytes).await?;
                Ok(cached)
            }
            .await;

            match result {
                Ok(cached) => {
                    progress.complete();
                    Ok(Resource::new(uri).resolved(cached.clone(), tokio::fs::metadata(cached).await?.len()))
                }
                Err(error) => {
                    progress.failed();
                    Err(error)
                }
            }
        })
    }
}
