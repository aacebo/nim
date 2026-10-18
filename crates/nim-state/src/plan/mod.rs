mod create;
mod delete;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{PgPool, PlanStorage};

#[derive(Copy, Clone)]
pub struct PlanService<'a> {
    store: PlanStorage<'a>,
}

impl<'a> PlanService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: PlanStorage::new(pool),
        }
    }
}
