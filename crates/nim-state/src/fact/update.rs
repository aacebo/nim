use crate::{Error, Fact, FactId, FactService};

#[derive(Debug, Default)]
pub struct FactUpdate {
    pub description: Option<String>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
    pub recalls: Option<u64>,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> FactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: FactId, value: FactUpdate) -> Result<Fact, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.description {
            row.description = value;
        }

        if let Some(value) = value.confidence {
            row.confidence = value;
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        if let Some(value) = value.recalls {
            row.recalls = value as i64;
        }

        if let Some(value) = value.recalled_at {
            row.recalled_at = Some(value);
        }

        Ok(self.store.update(row).await?.into())
    }
}
