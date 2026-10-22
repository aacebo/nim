use crate::{Annotation, AnnotationService, Error, MemoryId};

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_memory_id(&self, memory_id: MemoryId) -> Result<Vec<Annotation>, Error> {
        Ok(self
            .store
            .find_by_memory_id(memory_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
