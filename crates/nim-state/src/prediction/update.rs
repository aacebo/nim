use crate::{Error, Prediction, PredictionId, PredictionService};

#[derive(Debug, Default)]
pub struct PredictionUpdate {
    pub hypothesis: Option<String>,
    pub confidence: Option<f32>,
    pub deadline: Option<Option<chrono::DateTime<chrono::Utc>>>,
}

impl<'a> PredictionService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: PredictionId, value: PredictionUpdate) -> Result<Prediction, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.hypothesis {
            row.hypothesis = value;
        }

        if let Some(value) = value.confidence {
            row.confidence = value as f64;
        }

        if let Some(value) = value.deadline {
            row.deadline = value;
        }

        Ok(self.store.update(row).await?.into())
    }
}
