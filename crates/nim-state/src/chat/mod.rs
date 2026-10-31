mod create;
mod delete;
mod find_by_artifact_id;
mod find_by_entity_id;
mod find_by_id;
mod link;
mod search;
mod unlink;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct ChatService<'a> {
    storage: Storage<'a>,
}

impl<'a> ChatService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
