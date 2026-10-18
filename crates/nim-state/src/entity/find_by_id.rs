use crate::{Entity, EntityId, EntityService, Error};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: EntityId) -> Result<Entity, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
