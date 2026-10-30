use std::path::{Path, PathBuf};

use crate::Error;

use super::format::Format;
use super::uri::Uri;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Resource {
    pub uri: Uri,
    pub format: Format,
    pub path: Option<PathBuf>,
    pub size: u64,
}

impl Resource {
    pub fn new(uri: Uri) -> Self {
        let format = uri.format();
        let path = match &uri {
            Uri::Local(path) => Some(path.clone()),
            Uri::Http(_) => None,
        };

        Self {
            uri,
            format,
            path,
            size: 0,
        }
    }

    pub(crate) fn from_path(path: PathBuf) -> Result<Self, Error> {
        let metadata = std::fs::metadata(&path)?;
        let size = match metadata.is_dir() {
            true => directory_size(&path)?,
            false => metadata.len(),
        };
        let mut resource = Self::new(Uri::local(&path)).resolved(path, size);

        if metadata.is_dir() {
            resource.format = Format::Unknown;
        }

        Ok(resource)
    }

    pub fn local_path(&self) -> Result<&Path, Error> {
        self.path
            .as_deref()
            .ok_or_else(|| Error::custom("ai::resource", format!("{self} has no resolved local path")))
    }

    pub fn name(&self) -> &str {
        self.uri.name().unwrap_or("??")
    }

    pub fn resolved(mut self, path: PathBuf, size: u64) -> Self {
        self.path = Some(path);
        self.size = size;
        self
    }
}

impl From<Uri> for Resource {
    fn from(uri: Uri) -> Self {
        Self::new(uri)
    }
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}

fn directory_size(path: &Path) -> Result<u64, Error> {
    let mut size = 0u64;

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let bytes = if ty.is_dir() {
            directory_size(&entry.path())?
        } else {
            let metadata = std::fs::metadata(entry.path())?;

            match metadata.is_file() {
                true => metadata.len(),
                false => 0,
            }
        };

        size = size
            .checked_add(bytes)
            .ok_or_else(|| Error::custom("ai::resource", "directory size overflow"))?;
    }

    Ok(size)
}
