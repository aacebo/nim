mod create;
mod delete;
mod find_by_id;
mod find_by_memory_id;
mod link;
mod unlink;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct ArtifactService<'a> {
    storage: Storage<'a>,
}

impl<'a> ArtifactService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
