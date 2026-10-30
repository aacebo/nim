use std::path::{Path, PathBuf};

use crate::{
    Error,
    resources::{Asset, AssetData, Repository, Resource},
};

pub struct FileSystemRepository {
    root: PathBuf,
}

impl FileSystemRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Repository for FileSystemRepository {
    fn exists(&self, path: &Path) -> bool {
        self.root.join(path).exists()
    }

    fn get(&self, path: &Path) -> Result<Asset, Error> {
        let resource = self.resolve(path)?;
        let path = resource.local_path()?;

        match path.is_dir() {
            true => Ok(Asset::directory(path)),
            false => Ok(Asset::file(path)),
        }
    }

    fn read(&self, path: &Path) -> Result<AssetData, Error> {
        let resource = self.resolve(path)?;
        let path = resource.local_path()?;

        if !path.is_dir() {
            return Ok(AssetData::File(std::fs::read(path)?));
        }

        let mut assets = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            assets.push(match path.is_dir() {
                true => Asset::directory(path),
                false => Asset::file(path),
            });
        }

        Ok(AssetData::Directory(assets))
    }

    fn resolve(&self, path: &Path) -> Result<Resource, Error> {
        let path = self.root.join(path);

        match path.exists() {
            true => Resource::from_path(path),
            false => Err(Error::IO(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{} not found", path.display()),
            ))),
        }
    }
}
