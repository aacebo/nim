mod create;
mod delete;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, RecallStorage};

#[derive(Copy, Clone)]
pub struct RecallService<'a> {
    store: RecallStorage<'a>,
}

impl<'a> RecallService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: RecallStorage::new(pool),
        }
    }
}
