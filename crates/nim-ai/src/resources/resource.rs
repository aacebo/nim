use std::path::{Path, PathBuf};

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

    pub(crate) async fn from_path(path: PathBuf) -> nim_error::Result<Self> {
        let metadata = tokio::fs::metadata(&path).await?;
        let size = match metadata.is_dir() {
            true => directory_size(&path).await?,
            false => metadata.len(),
        };
        let mut resource = Self::new(Uri::local(&path)).resolved(path, size);

        if metadata.is_dir() {
            resource.format = Format::Unknown;
        }

        Ok(resource)
    }

    pub fn local_path(&self) -> nim_error::Result<&Path> {
        self.path
            .as_deref()
            .ok_or_else(|| nim_error::Diagnostic::message(format!("{self} has no resolved local path")))
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

async fn directory_size(path: &Path) -> nim_error::Result<u64> {
    let mut size = 0u64;
    let mut directories = vec![path.to_path_buf()];

    while let Some(path) = directories.pop() {
        let mut entries = tokio::fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let ty = entry.file_type().await?;

            if ty.is_dir() {
                directories.push(entry.path());
            } else {
                let metadata = tokio::fs::metadata(entry.path()).await?;

                if metadata.is_file() {
                    size = size
                        .checked_add(metadata.len())
                        .ok_or_else(|| nim_error::Diagnostic::message("directory size overflow"))?;
                }
            }
        }
    }

    Ok(size)
}
