mod create;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{MemoryStorage, PgPool};

#[derive(Copy, Clone)]
pub struct MemoryService<'a> {
    store: MemoryStorage<'a>,
}

impl<'a> MemoryService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: MemoryStorage::new(pool),
        }
    }
}
