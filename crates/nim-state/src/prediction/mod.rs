use nim_store::{PgPool, PredictionStorage};

#[derive(Copy, Clone)]
pub struct PredictionService<'a> {
    _store: PredictionStorage<'a>,
}

impl<'a> PredictionService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _store: PredictionStorage::new(pool),
        }
    }
}
