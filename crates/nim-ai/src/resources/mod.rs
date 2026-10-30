mod asset;
pub mod cache;
mod format;
mod resource;
mod uri;

use std::path::Path;

pub use asset::{Asset, AssetData, Directory as AssetDirectory, File as AssetFile};
pub use format::Format;
pub use resource::Resource;
pub use uri::Uri;

use crate::Error;
use crate::models::ModelId;

pub trait Repository: Send + Sync {
    fn exists(&self, path: &Path) -> bool;
    fn get(&self, path: &Path) -> Result<Asset, Error>;
    fn read(&self, path: &Path) -> Result<AssetData, Error>;
    fn resolve(&self, path: &Path) -> Result<Resource, Error>;
}

pub trait DataSource: Send + Sync {
    type Repository: Repository;

    fn load(&self, id: &ModelId) -> Result<Self::Repository, Error>;
}
