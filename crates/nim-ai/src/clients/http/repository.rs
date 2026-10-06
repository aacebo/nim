use std::path::{Path, PathBuf};

use crate::{
    Error,
    resources::{Asset, AssetData, Repository, Uri, cache},
};

pub struct Http {
    base: Uri,
}

impl Http {
    pub fn new(base: Uri) -> Self {
        Self { base }
    }

    fn cached(&self, file: &str) -> PathBuf {
        cache::dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("urls")
            .join(slug(&self.base.to_string()))
            .join(file)
    }
}

impl Repository for Http {
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
        let cached = self.cached(url.as_str());

        if cached.exists() {
            return Ok(cached);
        }

        let url = self.base.join(url.as_str())?.to_string();
        let response = reqwest::blocking::get(&url)?;
        let response = response.error_for_status()?;
        let bytes = response.bytes()?;

        if let Some(dir) = cached.parent() {
            std::fs::create_dir_all(dir)?;
        }

        std::fs::write(&cached, &bytes)?;
        Ok(cached)
    }
}

fn slug(base: &str) -> String {
    base.chars().map(|ch| if ch.is_alphanumeric() { ch } else { '-' }).collect()
}
