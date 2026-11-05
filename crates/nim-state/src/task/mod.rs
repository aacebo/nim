mod create;
mod create_event;
mod delete;
mod find_by_artifact_id;
mod find_by_chat_id;
mod find_by_id;
mod find_by_message_id;
mod find_by_model_id;
mod find_event_by_id;
mod find_events;
mod link;
mod unlink;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct TaskService<'a> {
    storage: Storage<'a>,
}

impl<'a> TaskService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
