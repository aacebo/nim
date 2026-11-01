mod create;
mod create_annotation;
mod delete;
mod delete_annotation;
mod find_annotation;
mod find_annotation_by_label;
mod find_annotations;
mod find_by_chat_id;
mod find_by_id;
mod find_by_parent_id;
mod search;
mod update;
mod update_annotation;

pub use create::*;
pub use create_annotation::*;
pub use update::*;
pub use update_annotation::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct MessageService<'a> {
    storage: Storage<'a>,
}

impl<'a> MessageService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
