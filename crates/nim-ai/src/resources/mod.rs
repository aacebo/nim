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

use crate::models::ModelId;

pub trait Repository: Send + Sync {
    fn exists<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, bool>;
    fn get<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<Asset>>;
    fn read<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<AssetData>>;
    fn resolve<'a>(&'a self, path: &'a Path) -> futures_util::future::BoxFuture<'a, nim_error::Result<Resource>>;
}

pub trait DataSource: Send + Sync {
    type Repository: Repository;

    fn load(&self, id: &ModelId) -> nim_error::Result<Self::Repository>;
}
