mod annotation;
mod artifact;
mod entity;
mod error;
mod fact;
mod goal;
mod memory;
mod plan;
mod prediction;
mod types;

pub use annotation::*;
pub use artifact::*;
pub use entity::*;
pub use error::*;
pub use fact::*;
pub use goal::*;
pub use memory::*;
pub use plan::*;
pub use prediction::*;
pub use types::*;

#[derive(Copy, Clone)]
pub struct Services<'a> {
    _annotations: AnnotationService<'a>,
    _artifacts: ArtifactService<'a>,
    _entities: EntityService<'a>,
    _facts: FactService<'a>,
    _goals: GoalService<'a>,
    _memories: MemoryService<'a>,
    _plans: PlanService<'a>,
    _predictions: PredictionService<'a>,
}

impl<'a> Services<'a> {
    pub fn new(pool: &'a nim_store::PgPool) -> Self {
        Self {
            _annotations: AnnotationService::new(pool),
            _artifacts: ArtifactService::new(pool),
            _entities: EntityService::new(pool),
            _facts: FactService::new(pool),
            _goals: GoalService::new(pool),
            _memories: MemoryService::new(pool),
            _plans: PlanService::new(pool),
            _predictions: PredictionService::new(pool),
        }
    }

    pub fn annotations(&self) -> AnnotationService<'a> {
        self._annotations
    }

    pub fn artifacts(&self) -> ArtifactService<'a> {
        self._artifacts
    }

    pub fn entities(&self) -> EntityService<'a> {
        self._entities
    }

    pub fn facts(&self) -> FactService<'a> {
        self._facts
    }

    pub fn goals(&self) -> GoalService<'a> {
        self._goals
    }

    pub fn memories(&self) -> MemoryService<'a> {
        self._memories
    }

    pub fn plans(&self) -> PlanService<'a> {
        self._plans
    }

    pub fn predictions(&self) -> PredictionService<'a> {
        self._predictions
    }
}
