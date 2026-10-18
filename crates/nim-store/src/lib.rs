mod annotation;
mod entity;
mod fact;
mod goal;
mod memory;
mod plan;
mod prediction;
mod types;

pub use annotation::*;
pub use entity::*;
pub use fact::*;
pub use goal::*;
pub use memory::*;
pub use plan::*;
pub use prediction::*;
pub use types::*;

pub use pgvector::Vector;
pub use sqlx::Error;
pub use sqlx::PgPool;
pub use sqlx::migrate;
pub use sqlx::types::Json;

#[derive(Copy, Clone)]
pub struct Storage<'a> {
    _annotations: AnnotationStorage<'a>,
    _entities: EntityStorage<'a>,
    _goals: GoalStorage<'a>,
    _memories: MemoryStorage<'a>,
    _facts: FactStorage<'a>,
    _plans: PlanStorage<'a>,
    _predictions: PredictionStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self {
            _annotations: AnnotationStorage::new(pool),
            _entities: EntityStorage::new(pool),
            _goals: GoalStorage::new(pool),
            _memories: MemoryStorage::new(pool),
            _facts: FactStorage::new(pool),
            _plans: PlanStorage::new(pool),
            _predictions: PredictionStorage::new(pool),
        }
    }

    pub fn annotations(&self) -> AnnotationStorage<'a> {
        self._annotations
    }

    pub fn entities(&self) -> EntityStorage<'a> {
        self._entities
    }

    pub fn goals(&self) -> GoalStorage<'a> {
        self._goals
    }

    pub fn memories(&self) -> MemoryStorage<'a> {
        self._memories
    }

    pub fn facts(&self) -> FactStorage<'a> {
        self._facts
    }

    pub fn plans(&self) -> PlanStorage<'a> {
        self._plans
    }

    pub fn predictions(&self) -> PredictionStorage<'a> {
        self._predictions
    }
}
