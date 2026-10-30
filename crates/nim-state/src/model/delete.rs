use crate::{Error, ModelId, ModelService};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: ModelId) -> Result<(), Error> {
        Ok(self.storage.models().delete(id.into()).await?)
    }
}
