use nim_store::{PgPool, PlanStorage};

#[derive(Copy, Clone)]
pub struct PlanService<'a> {
    _store: PlanStorage<'a>,
}

impl<'a> PlanService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _store: PlanStorage::new(pool),
        }
    }
}
