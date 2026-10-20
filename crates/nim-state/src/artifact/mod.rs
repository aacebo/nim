mod create;
mod delete;
mod find_by_id;
mod find_by_memory_id;
mod link;
mod unlink;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{ArtifactStorage, PgPool};

#[derive(Copy, Clone)]
pub struct ArtifactService<'a> {
    store: ArtifactStorage<'a>,
}

impl<'a> ArtifactService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: ArtifactStorage::new(pool),
        }
    }
}
