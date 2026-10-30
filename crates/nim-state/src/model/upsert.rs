use nim_store::ModelRow;

use crate::{Error, Model, ModelCreate, ModelService};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn upsert(&self, value: ModelCreate) -> Result<Model, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .models()
            .upsert(ModelRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                ty: value.ty,
                name: value.name,
                tasks: value.tasks,
                architectures: value.architectures,
                source: nim_store::Json(value.source),
                weights: nim_store::Json(value.weights),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
