use crate::{Error, Model, ModelId, ModelService};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: ModelId) -> Result<Model, Error> {
        Ok(self.storage.models().find_by_id(id.into()).await?.into())
    }
}
