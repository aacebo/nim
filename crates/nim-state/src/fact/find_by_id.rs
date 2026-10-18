use crate::{Error, Fact, FactId, FactService};

impl<'a> FactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: FactId) -> Result<Fact, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
