mod create;
mod delete;
mod find_by_id;
mod link;
mod relate;
mod unlink;
mod unrelate;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct EntityService<'a> {
    storage: Storage<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
