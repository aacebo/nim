use nim_store::EntityRelationRow;

use crate::{EntityId, EntityRelation, EntityRelationType, EntityService, Error};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn relate(
        &self,
        source_id: EntityId,
        target_id: EntityId,
        ty: EntityRelationType,
    ) -> Result<EntityRelation, Error> {
        let row = self
            .storage
            .entity_relations()
            .create(EntityRelationRow {
                source_id: source_id.into(),
                target_id: target_id.into(),
                ty: ty.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
