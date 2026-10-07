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

pub use sqlx::Error;
pub use sqlx::PgPool;
pub use sqlx::migrate;

pub struct Storage<'a> {
    pub annotations: AnnotationStorage<'a>,
    pub entities: EntityStorage<'a>,
    pub goals: GoalStorage<'a>,
    pub memories: MemoryStorage<'a>,
    pub facts: FactStorage<'a>,
    pub plans: PlanStorage<'a>,
    pub predictions: PredictionStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self {
            annotations: AnnotationStorage::new(pool),
            entities: EntityStorage::new(pool),
            goals: GoalStorage::new(pool),
            memories: MemoryStorage::new(pool),
            facts: FactStorage::new(pool),
            plans: PlanStorage::new(pool),
            predictions: PredictionStorage::new(pool),
        }
    }
}
