mod create;
mod create_weight;
mod delete;
mod delete_weight;
mod find_by_id;
mod find_by_name;
mod find_weight;
mod find_weights;
mod update;
mod update_weight;
mod upsert;

pub use create::*;
pub use create_weight::*;
pub use update::*;
pub use update_weight::*;

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
