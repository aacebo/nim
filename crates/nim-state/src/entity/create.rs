use nim_store::EntityRow;

use crate::{Entity, EntityService, EntityType, Error};

#[derive(Debug)]
pub struct EntityCreate {
    pub ty: EntityType,
    pub name: String,
}

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: EntityCreate) -> Result<Entity, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(EntityRow {
                id: uuid::Uuid::now_v7(),
                ty: value.ty.into(),
                version: 0,
                name: value.name,
                summary: None,
                confidence: 0.0,
                embedding: None,
                recalls: 0,
                recalled_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
