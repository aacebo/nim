use crate::{EntityId, EntityRelationType, EntityService};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unrelate(&self, source_id: EntityId, target_id: EntityId, ty: EntityRelationType) -> nim_error::Result<()> {
        self.storage
            .entity_relations()
            .delete((source_id.into(), target_id.into(), ty.to_string()))
            .await?;

        Ok(())
    }
}
