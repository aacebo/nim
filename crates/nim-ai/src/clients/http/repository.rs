use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    Error,
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
        let filename = path
            .to_str()
            .ok_or_else(|| Error::custom("url::parse", format!("{path:?} is not utf-8")))?;
        let uri = self.base.join(filename)?;
        let key = format!("{:x}", Sha256::digest(uri.to_string().as_bytes()));
        let cached = cache::dir().unwrap_or_else(std::env::temp_dir).join("urls").join(key);
        let mut progress = Download::new(&self.base.to_string(), filename);

        if cached.exists() {
            progress.cached();
            return Ok(Resource::new(uri).resolved(cached.clone(), std::fs::metadata(cached)?.len()));
        }

        progress.start();

        let result = (|| -> Result<PathBuf, Error> {
            let url = uri.to_string();
            let mut response = reqwest::blocking::get(&url)?.error_for_status()?;
            progress.reset(response.content_length());
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 65536];

            loop {
                let read = response.read(&mut buffer)?;

                if read == 0 {
                    break;
                }

                bytes.extend_from_slice(&buffer[..read]);
                progress.update(read);
            }

            if let Some(dir) = cached.parent() {
                std::fs::create_dir_all(dir)?;
            }

            std::fs::write(&cached, &bytes)?;
            Ok(cached)
        })();

        match result {
            Ok(cached) => {
                progress.complete();
                Ok(Resource::new(uri).resolved(cached.clone(), std::fs::metadata(cached)?.len()))
            }
            Err(error) => {
                progress.failed();
                Err(error)
            }
        }
    }
}
