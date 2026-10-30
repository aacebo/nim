use std::path::PathBuf;

use crate::Error;
use crate::models::ModelId;
use crate::resources::DataSource;

use super::FileSystemRepository;

pub struct FileSystem {
    root: PathBuf,
}

impl FileSystem {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl DataSource for FileSystem {
    type Repository = FileSystemRepository;

    fn load(&self, id: &ModelId) -> Result<Self::Repository, Error> {
        Ok(FileSystemRepository::new(self.root.join(id.to_string())))
    }
}
