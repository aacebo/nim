mod create;
mod delete;
mod find_by_id;
mod find_by_memory_id;
mod find_by_memory_label;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{AnnotationStorage, PgPool};

#[derive(Copy, Clone)]
pub struct AnnotationService<'a> {
    store: AnnotationStorage<'a>,
}

impl<'a> AnnotationService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: AnnotationStorage::new(pool),
        }
    }
}
