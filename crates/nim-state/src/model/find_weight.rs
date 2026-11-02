use crate::{Error, ModelId, ModelService, ModelWeight, ModelWeightId};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_weight(&self, model_id: ModelId, id: ModelWeightId) -> Result<ModelWeight, Error> {
        self.storage
            .model_weights()
            .find_by_id(model_id.into(), id.into())
            .await?
            .try_into()
    }
}
