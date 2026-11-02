use nim_store::ModelWeightRow;

use crate::{Error, ModelId, ModelService, ModelWeight, ModelWeightContent, ModelWeightId};

#[derive(Debug)]
pub struct ModelWeightCreate {
    pub name: String,
    pub content: ModelWeightContent,
}

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create_weight(&self, model_id: ModelId, value: ModelWeightCreate) -> Result<ModelWeight, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .model_weights()
            .create(ModelWeightRow {
                id: ModelWeightId::new().into(),
                model_id: model_id.into(),
                version: 0,
                name: value.name,
                content: serde_json::to_value(value.content)?,
                created_at: now,
                updated_at: now,
            })
            .await?;

        row.try_into()
    }
}
