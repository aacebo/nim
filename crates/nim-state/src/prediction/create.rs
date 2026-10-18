use nim_store::PredictionRow;

use crate::{Error, Prediction, PredictionService};

#[derive(Debug)]
pub struct PredictionCreate {
    pub hypothesis: String,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> PredictionService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: PredictionCreate) -> Result<Prediction, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(PredictionRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                hypothesis: value.hypothesis,
                confidence: 0.0,
                deadline: value.deadline,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
