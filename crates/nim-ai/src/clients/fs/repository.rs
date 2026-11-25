use std::path::{Path, PathBuf};

use crate::resources::{Asset, AssetData, Repository, Resource};

pub struct FileSystemRepository {
    root: PathBuf,
}

impl FileSystemRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Repository for FileSystemRepository {
    fn exists<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, bool> {
        Box::pin(async move { tokio::fs::try_exists(self.root.join(path)).await.unwrap_or(false) })
    }

    fn get<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<Asset>> {
        Box::pin(async move {
            let resource = self.resolve(path).await?;
            let path = resource.local_path()?;

            match tokio::fs::metadata(path).await?.is_dir() {
                true => Ok(Asset::directory(path)),
                false => Ok(Asset::file(path)),
            }
        })
    }

    fn read<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<AssetData>> {
        Box::pin(async move {
            let resource = self.resolve(path).await?;
            let path = resource.local_path()?;

            if !tokio::fs::metadata(path).await?.is_dir() {
                return Ok(AssetData::File(tokio::fs::read(path).await?));
            }

            let mut assets = Vec::new();
            let mut entries = tokio::fs::read_dir(path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                assets.push(match entry.file_type().await?.is_dir() {
                    true => Asset::directory(path),
                    false => Asset::file(path),
                });
            }

            Ok(AssetData::Directory(assets))
        })
    }

    fn resolve<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<Resource>> {
        Box::pin(async move {
            let path = self.root.join(path);

            match tokio::fs::try_exists(&path).await? {
                true => Resource::from_path(path).await,
                false => Err(nim_error::Diagnostic::from(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} not found", path.display()),
                ))),
            }
        })
    }
}
