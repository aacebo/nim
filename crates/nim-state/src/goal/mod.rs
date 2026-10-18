mod create;
mod delete;
mod find_by_id;
mod update;

pub use create::*;
pub use update::*;

use nim_store::{GoalStorage, PgPool};

#[derive(Copy, Clone)]
pub struct GoalService<'a> {
    store: GoalStorage<'a>,
}

impl<'a> GoalService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: GoalStorage::new(pool),
        }
    }
}
