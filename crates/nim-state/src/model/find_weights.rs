use crate::{Error, ModelId, ModelService, ModelWeight};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_weights(&self, model_id: ModelId) -> Result<Vec<ModelWeight>, Error> {
        let rows = self.storage.model_weights().find_by_model_id(model_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
