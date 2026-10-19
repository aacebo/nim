use crate::{Entity, EntityId, EntityService, EntityType, Error};

#[derive(Debug, Default)]
pub struct EntityUpdate {
    pub ty: Option<EntityType>,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
    pub recalls: Option<u64>,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: EntityId, value: EntityUpdate) -> Result<Entity, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.ty {
            row.ty = value.into();
        }

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.summary {
            row.summary = Some(value);
        }

        if let Some(value) = value.confidence {
            row.confidence = value as f64;
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        if let Some(value) = value.recalls {
            row.recalls = value as i64;
        }

        if let Some(value) = value.recalled_at {
            row.recalled_at = Some(value);
        }

        row.version += 1;
        Ok(self.store.update(row).await?.into())
    }
}
