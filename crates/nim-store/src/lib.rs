mod annotation;
mod entity;
mod goal;
mod memory;
mod observation;
mod plan;
mod prediction;
pub mod types;

pub use annotation::*;
pub use entity::*;
pub use goal::*;
pub use memory::*;
pub use observation::*;
pub use plan::*;
pub use prediction::*;

pub struct Storage<'a> {
    pub annotations: AnnotationStorage<'a>,
    pub entities: EntityStorage<'a>,
    pub goals: GoalStorage<'a>,
    pub memories: MemoryStorage<'a>,
    pub observations: ObservationStorage<'a>,
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
            observations: ObservationStorage::new(pool),
            plans: PlanStorage::new(pool),
            predictions: PredictionStorage::new(pool),
        }
    }
}
