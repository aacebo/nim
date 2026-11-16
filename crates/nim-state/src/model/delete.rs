use crate::{ModelId, ModelService};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: ModelId) -> nim_error::Result<()> {
        Ok(self.storage.models().delete(id.into()).await?)
    }
}
