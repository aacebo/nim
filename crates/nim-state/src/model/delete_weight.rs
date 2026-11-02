use crate::{Error, ModelId, ModelService, ModelWeightId};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete_weight(&self, model_id: ModelId, id: ModelWeightId) -> Result<(), Error> {
        Ok(self.storage.model_weights().delete(model_id.into(), id.into()).await?)
    }
}
