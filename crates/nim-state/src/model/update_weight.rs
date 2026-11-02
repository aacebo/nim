use crate::{Error, ModelId, ModelService, ModelWeight, ModelWeightContent, ModelWeightId};

#[derive(Debug, Default)]
pub struct ModelWeightUpdate {
    pub name: Option<String>,
    pub content: Option<ModelWeightContent>,
}

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update_weight(
        &self,
        model_id: ModelId,
        id: ModelWeightId,
        value: ModelWeightUpdate,
    ) -> Result<ModelWeight, Error> {
        let mut row = self.storage.model_weights().find_by_id(model_id.into(), id.into()).await?;

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.content {
            row.content = serde_json::to_value(value)?;
        }

        self.storage.model_weights().update(row).await?.try_into()
    }
}
