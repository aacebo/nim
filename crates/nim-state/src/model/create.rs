use nim_store::ModelRow;

use crate::{Error, Model, ModelService, ModelSource};

#[derive(Debug)]
pub struct ModelCreate {
    pub ty: String,
    pub name: String,
    pub tasks: Vec<String>,
    pub architectures: Vec<String>,
    pub source: ModelSource,
}

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: ModelCreate) -> Result<Model, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .models()
            .create(ModelRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                ty: value.ty,
                name: value.name,
                tasks: value.tasks,
                architectures: value.architectures,
                source: nim_store::Json(value.source),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
