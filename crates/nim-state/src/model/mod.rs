mod create;
mod delete;
mod find_by_id;
mod find_by_name;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct ModelService<'a> {
    storage: Storage<'a>,
}

impl<'a> ModelService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
