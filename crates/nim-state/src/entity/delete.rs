use crate::{EntityId, EntityService, Error};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: EntityId) -> Result<(), Error> {
        Ok(self.storage.entities().delete(id.into()).await?)
    }
}
