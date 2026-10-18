use nim_store::FactRow;

use crate::{Error, Fact, FactService};

#[derive(Debug)]
pub struct FactCreate {
    pub description: String,
}

impl<'a> FactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: FactCreate) -> Result<Fact, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(FactRow {
                id: uuid::Uuid::now_v7(),
                description: value.description,
                confidence: 0.0,
                embedding: None,
                recalls: 0,
                recalled_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
