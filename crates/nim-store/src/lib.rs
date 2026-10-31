mod annotation;
mod artifact;
mod entity;
mod memory;
mod model;
mod recall;
mod search;
mod task;
mod types;

pub use annotation::*;
pub use artifact::*;
pub use entity::*;
pub use memory::*;
pub use model::*;
pub use recall::*;
pub use search::*;
pub use task::*;
pub use types::*;

pub use pgvector::Vector;
pub use sqlx::Error;
pub use sqlx::PgPool;
pub use sqlx::migrate;
pub use sqlx::types::Json;

#[derive(Copy, Clone)]
pub struct Storage<'a> {
    _annotations: AnnotationStorage<'a>,
    _artifacts: ArtifactStorage<'a>,
    _memory_artifacts: MemoryArtifactsStorage<'a>,
    _entities: EntityStorage<'a>,
    _entity_relations: EntityRelationsStorage<'a>,
    _entity_memories: EntityMemoriesStorage<'a>,
    _memories: MemoryStorage<'a>,
    _models: ModelStorage<'a>,
    _memory_relations: MemoryRelationsStorage<'a>,
    _recalls: RecallStorage<'a>,
    _artifact_tasks: ArtifactTasksStorage<'a>,
    _model_tasks: ModelTasksStorage<'a>,
    _tasks: TaskStorage<'a>,
    _memory_recalls: MemoryRecallsStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self {
            _annotations: AnnotationStorage::new(pool),
            _artifacts: ArtifactStorage::new(pool),
            _memory_artifacts: MemoryArtifactsStorage::new(pool),
            _entities: EntityStorage::new(pool),
            _entity_relations: EntityRelationsStorage::new(pool),
            _entity_memories: EntityMemoriesStorage::new(pool),
            _memories: MemoryStorage::new(pool),
            _models: ModelStorage::new(pool),
            _memory_relations: MemoryRelationsStorage::new(pool),
            _recalls: RecallStorage::new(pool),
            _artifact_tasks: ArtifactTasksStorage::new(pool),
            _model_tasks: ModelTasksStorage::new(pool),
            _tasks: TaskStorage::new(pool),
            _memory_recalls: MemoryRecallsStorage::new(pool),
        }
    }

    pub fn annotations(&self) -> AnnotationStorage<'a> {
        self._annotations
    }

    pub fn artifacts(&self) -> ArtifactStorage<'a> {
        self._artifacts
    }

    pub fn memory_artifacts(&self) -> MemoryArtifactsStorage<'a> {
        self._memory_artifacts
    }

    pub fn entities(&self) -> EntityStorage<'a> {
        self._entities
    }

    pub fn entity_relations(&self) -> EntityRelationsStorage<'a> {
        self._entity_relations
    }

    pub fn entity_memories(&self) -> EntityMemoriesStorage<'a> {
        self._entity_memories
    }

    pub fn memories(&self) -> MemoryStorage<'a> {
        self._memories
    }

    pub fn memory_relations(&self) -> MemoryRelationsStorage<'a> {
        self._memory_relations
    }

    pub fn recalls(&self) -> RecallStorage<'a> {
        self._recalls
    }

    pub fn memory_recalls(&self) -> MemoryRecallsStorage<'a> {
        self._memory_recalls
    }

    pub fn models(&self) -> ModelStorage<'a> {
        self._models
    }

    pub fn tasks(&self) -> TaskStorage<'a> {
        self._tasks
    }

    pub fn model_tasks(&self) -> ModelTasksStorage<'a> {
        self._model_tasks
    }

    pub fn artifact_tasks(&self) -> ArtifactTasksStorage<'a> {
        self._artifact_tasks
    }
}
