use nim_store::{MemoryStorage, PgPool};

use crate::{Context, Error, Memory};

pub struct MemoryService<'a> {
    store: MemoryStorage<'a>,
}

impl<'a> MemoryService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            store: MemoryStorage::new(pool),
        }
    }
}

impl<'a> MemoryService<'a> {
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Memory, Error> {
        let row = self.store.find_by_id(id).await?;

        Ok(Memory {
            id: row.id.into(),
            version: row.version.into(),
            ty: row.ty.into(),
            sources: Default::default(),
            relations: Default::default(),
            contexts: row.contexts.into_inner().into_iter().map(Context::from).collect(),
            annotations: Default::default(),
            salience: row.salience,
            strength: row.strength,
            confidence: row.confidence,
            recalls: row.recalls as u64,
            description: row.description,
            summary: row.summary,
            embedding: row.embedding.map(|v| v.to_vec().into()),
            recalled_at: row.recalled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
