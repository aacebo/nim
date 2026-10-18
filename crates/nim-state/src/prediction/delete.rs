use crate::{Error, PredictionId, PredictionService};

impl<'a> PredictionService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: PredictionId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
