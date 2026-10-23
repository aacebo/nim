use nim_store::MemoryRow;

use crate::{Context, Error, Memory, MemoryService, MemoryType, Source};

#[derive(Debug)]
pub struct MemoryCreate<'a> {
    pub ty: MemoryType,
    pub sources: &'a [Source],
    pub contexts: &'a [Context],
    pub summary: Option<&'a str>,
    pub confidence: f32,
    pub embedding: Option<&'a [f32]>,
}

impl<'a> Default for MemoryCreate<'a> {
    fn default() -> Self {
        Self {
            ty: MemoryType::Working,
            sources: Default::default(),
            contexts: Default::default(),
            summary: None,
            confidence: 0.0,
            embedding: None,
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
                summary: value.summary.map(|v| v.to_string()),
                embedding: value.embedding.map(|v| v.to_vec().into()),
                recalled_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(Memory {
            id: row.id.into(),
            version: Default::default(),
            ty: row.ty.into(),
            sources: row.sources.iter().map(|v| (*v).into()).collect(),
            artifacts: Default::default(),
            contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
            relations: Default::default(),
            salience: row.salience,
            strength: row.strength,
            confidence: row.confidence,
            recalls: row.recalls as u64,
            summary: row.summary,
            embedding: row.embedding.map(|v| v.into()),
            recalled_at: row.recalled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
