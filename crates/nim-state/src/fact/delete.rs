use crate::{Error, FactId, FactService};

impl<'a> FactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: FactId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
