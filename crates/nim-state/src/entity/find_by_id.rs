use crate::{Entity, EntityId, EntityService};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: EntityId) -> nim_error::Result<Entity> {
        Ok(self.storage.entities().find_by_id(id.into()).await?.into())
    }
}
