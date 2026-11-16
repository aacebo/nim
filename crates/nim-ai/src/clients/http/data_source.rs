use crate::models::ModelId;
use crate::resources::{DataSource, Uri};

use super::HttpRepository;

pub struct Http {
    base: Uri,
}

impl Http {
    pub fn new(base: Uri) -> Self {
        Self { base }
    }
}

impl DataSource for Http {
    type Repository = HttpRepository;

    fn load(&self, id: &ModelId) -> nim_error::Result<Self::Repository> {
        Ok(HttpRepository::new(self.base.join(&id.to_string())?))
    }
}
