use nim_store::{EntityStorage, PgPool};

#[derive(Copy, Clone)]
pub struct EntityService<'a> {
    _store: EntityStorage<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _store: EntityStorage::new(pool),
        }
    }
}
