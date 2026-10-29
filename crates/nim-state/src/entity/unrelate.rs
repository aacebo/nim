use crate::{EntityId, EntityRelationType, EntityService, Error};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unrelate(&self, source_id: EntityId, target_id: EntityId, ty: EntityRelationType) -> Result<(), Error> {
        self.storage
            .entity_relations()
            .delete((source_id.into(), target_id.into(), ty.into()))
            .await?;

        Ok(())
    }
}
