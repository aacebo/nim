mod create;
mod delete;
mod find_by_id;
mod relate;
mod search;
mod unrelate;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct MemoryService<'a> {
    storage: Storage<'a>,
}

impl<'a> MemoryService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
