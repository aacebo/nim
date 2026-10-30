mod create;
mod delete;
mod find_by_id;
mod link;
mod unlink;

pub use create::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct RecallService<'a> {
    storage: Storage<'a>,
}

impl<'a> RecallService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
