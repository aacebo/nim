use nim_store::MemoryRow;

use crate::{Context, Error, Memory, MemoryService, MemoryType, Source};

#[derive(Debug)]
pub struct MemoryCreate<'a> {
    pub ty: MemoryType,
    pub sources: &'a [Source],
    pub contexts: &'a [Context],
    pub confidence: f32,
}

impl<'a> Default for MemoryCreate<'a> {
    fn default() -> Self {
        Self {
            ty: MemoryType::Working,
            sources: Default::default(),
            contexts: Default::default(),
            confidence: 0.0,
        }
    }
}

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create<'b>(&self, value: MemoryCreate<'b>) -> Result<Memory, Error> {
        let now = chrono::Utc::now();
        let sources = value.sources.iter().map(|v| (*v).into()).collect();
        let contexts = value.contexts.iter().map(|c| c.clone().into()).collect();
        let row = self
            .storage
            .memories()
            .create(MemoryRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                ty: value.ty.into(),
                sources: nim_store::Json(sources),
                contexts: nim_store::Json(contexts),
                salience: 0.0,
                strength: 0.0,
                confidence: value.confidence,
                recalls: 0,
                recalled_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(Memory {
            id: row.id.into(),
            version: row.version.into(),
            ty: row.ty.into(),
            sources: row.sources.iter().map(|v| (*v).into()).collect(),
            artifacts: Default::default(),
            contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
            relations: Default::default(),
            salience: row.salience,
            strength: row.strength,
            confidence: row.confidence,
            recalls: row.recalls as u64,
            recalled_at: row.recalled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
