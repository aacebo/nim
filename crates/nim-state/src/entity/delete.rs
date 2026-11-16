use crate::{EntityId, EntityService};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: EntityId) -> nim_error::Result<()> {
        Ok(self.storage.entities().delete(id.into()).await?)
    }
}
