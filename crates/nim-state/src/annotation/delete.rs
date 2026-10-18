use crate::{AnnotationId, AnnotationService, Error};

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: AnnotationId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
