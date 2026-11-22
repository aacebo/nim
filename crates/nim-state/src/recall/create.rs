use nim_store::RecallRow;

use crate::{Recall, RecallService, RecallSource, RecallTrigger};

#[derive(Debug)]
pub struct RecallCreate {
    pub trigger: RecallTrigger,
    pub source: RecallSource,
    pub query: Option<String>,
}

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: RecallCreate) -> nim_error::Result<Recall> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .recalls()
            .create(RecallRow {
                id: uuid::Uuid::now_v7(),
                trigger: value.trigger.to_string(),
                source: value.source.to_string(),
                query: value.query,
                created_at: now,
            })
            .await?;

        row.try_into()
    }
}
