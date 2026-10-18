use nim_store::{GoalStorage, PgPool};

#[derive(Copy, Clone)]
pub struct GoalService<'a> {
    _store: GoalStorage<'a>,
}

impl<'a> GoalService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _store: GoalStorage::new(pool),
        }
    }
}
