use crate::{Annotation, AnnotationService, Error, Label, MemoryId};

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_memory_label(&self, memory_id: MemoryId, label: Label, text: &str) -> Result<Option<Annotation>, Error> {
        Ok(self
            .store
            .find_by_memory_label(memory_id.into(), label.into(), text)
            .await?
            .map(Into::into))
    }
}
