use crate::{EntityId, EntityService, Error};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: EntityId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
