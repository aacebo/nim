use crate::{Annotation, AnnotationId, AnnotationService, Error};

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: AnnotationId) -> Result<Annotation, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
