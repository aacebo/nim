use nim_store::{AnnotationStorage, PgPool};

#[derive(Copy, Clone)]
pub struct AnnotationService<'a> {
    _store: AnnotationStorage<'a>,
}

impl<'a> AnnotationService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _store: AnnotationStorage::new(pool),
        }
    }
}
