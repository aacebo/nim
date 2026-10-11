use nim_store::MemoryRow;

use crate::{Context, Error, Memory, MemoryService, MemoryType};

#[derive(Debug)]
pub struct MemoryCreate<'a> {
    pub ty: MemoryType,
    pub contexts: &'a [Context],
    pub description: String,
}

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create<'b>(&self, value: MemoryCreate<'b>) -> Result<Memory, Error> {
        let now = chrono::Utc::now();
        let contexts = value.contexts.iter().map(|c| c.clone().into()).collect();
        let row = self
            .store
            .create(MemoryRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                ty: value.ty.into(),
                contexts: nim_store::Json(contexts),
                salience: 0.0,
                strength: 0.0,
                confidence: 0.0,
                recalls: 0,
                description: value.description,
                summary: None,
                embedding: None,
                recalled_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(Memory {
            id: row.id.into(),
            version: Default::default(),
            ty: row.ty.into(),
            contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
            sources: Default::default(),
            relations: Default::default(),
            annotations: Default::default(),
            salience: row.salience,
            strength: row.strength,
            confidence: row.confidence,
            recalls: row.recalls as u64,
            description: row.description,
            summary: row.summary,
            embedding: row.embedding.map(|v| v.into()),
            recalled_at: row.recalled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
