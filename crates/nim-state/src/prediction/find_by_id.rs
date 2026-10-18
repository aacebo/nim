use crate::{Error, Prediction, PredictionId, PredictionService};

impl<'a> PredictionService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: PredictionId) -> Result<Prediction, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
