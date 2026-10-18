mod create;
mod delete;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, PredictionStorage};

#[derive(Copy, Clone)]
pub struct PredictionService<'a> {
    store: PredictionStorage<'a>,
}

impl<'a> PredictionService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: PredictionStorage::new(pool),
        }
    }
}
