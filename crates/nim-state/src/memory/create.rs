use nim_store::MemoryRow;

use crate::{Context, Embedding, Memory, MemoryService, MemoryType, Source};

#[derive(Debug)]
pub struct MemoryCreate<'a> {
    pub ty: MemoryType,
    pub sources: &'a [Source],
    pub contexts: &'a [Context],
    pub summary: Option<String>,
    pub embedding: Option<Embedding>,
    pub confidence: f32,
}

impl<'a> Default for MemoryCreate<'a> {
    fn default() -> Self {
        Self {
            ty: MemoryType::Working,
            sources: Default::default(),
            contexts: Default::default(),
            summary: None,
            embedding: None,
            confidence: 0.0,
        }
    }
}

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create<'b>(&self, value: MemoryCreate<'b>) -> nim_error::Result<Memory> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .memories()
            .create(MemoryRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                ty: value.ty.to_string(),
                sources: serde_json::to_value(value.sources)?,
                contexts: serde_json::to_value(value.contexts)?,
                summary: value.summary,
                embedding: value.embedding.map(Into::into),
                salience: 0.0,
                strength: 0.0,
                confidence: value.confidence,
                recalls: 0,
                recalled_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        row.try_into()
    }
}
