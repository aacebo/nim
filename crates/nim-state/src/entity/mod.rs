mod create;
mod delete;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{EntityStorage, PgPool};

#[derive(Copy, Clone)]
pub struct EntityService<'a> {
    store: EntityStorage<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: EntityStorage::new(pool),
        }
    }
}
