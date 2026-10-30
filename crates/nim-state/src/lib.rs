mod annotation;
mod artifact;
mod entity;
mod error;
mod memory;
mod model;
mod recall;
mod types;

pub use annotation::*;
pub use artifact::*;
pub use entity::*;
pub use error::*;
pub use memory::*;
pub use model::*;
pub use recall::*;
pub use types::*;

#[derive(Copy, Clone)]
pub struct Services<'a> {
    _annotations: AnnotationService<'a>,
    _artifacts: ArtifactService<'a>,
    _entities: EntityService<'a>,
    _memories: MemoryService<'a>,
    _models: ModelService<'a>,
    _recalls: RecallService<'a>,
}

impl<'a> Services<'a> {
    pub fn new(pool: &'a nim_store::PgPool) -> Self {
        Self {
            _annotations: AnnotationService::new(pool),
            _artifacts: ArtifactService::new(pool),
            _entities: EntityService::new(pool),
            _memories: MemoryService::new(pool),
            _models: ModelService::new(pool),
            _recalls: RecallService::new(pool),
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

    pub fn memories(&self) -> MemoryService<'a> {
        self._memories
    }

    pub fn recalls(&self) -> RecallService<'a> {
        self._recalls
    }

    pub fn models(&self) -> ModelService<'a> {
        self._models
    }
}
