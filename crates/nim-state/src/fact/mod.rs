use nim_store::{FactStorage, PgPool};

#[derive(Copy, Clone)]
pub struct FactService<'a> {
    _store: FactStorage<'a>,
}

impl<'a> FactService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _store: FactStorage::new(pool),
        }
    }
}
