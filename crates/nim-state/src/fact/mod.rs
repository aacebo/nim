mod create;
mod delete;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{FactStorage, PgPool};

#[derive(Copy, Clone)]
pub struct FactService<'a> {
    store: FactStorage<'a>,
}

impl<'a> FactService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: FactStorage::new(pool),
        }
    }
}
