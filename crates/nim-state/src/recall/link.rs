use nim_store::MemoryRecallRow;

use crate::{Error, MemoryId, MemoryRecall, RecallId, RecallService};

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link(&self, memory_id: MemoryId, recall_id: RecallId, score: f32) -> Result<MemoryRecall, Error> {
        let row = self
            .storage
            .memory_recalls()
            .create(MemoryRecallRow {
                memory_id: memory_id.into(),
                recall_id: recall_id.into(),
                score,
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
