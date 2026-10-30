use crate::{Error, Memory, MemoryId, MemoryService, Source};

#[derive(Debug, Default)]
pub struct MemoryUpdate<'a> {
    pub sources: Option<&'a [Source]>,
    pub salience: Option<f32>,
    pub strength: Option<f32>,
    pub confidence: Option<f32>,
    pub recalls: Option<u64>,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update<'b>(&self, id: MemoryId, value: MemoryUpdate<'b>) -> Result<Memory, Error> {
        let mut row = self.storage.memories().find_by_id(id.into()).await?;

        if let Some(v) = value.sources {
            row.sources = nim_store::Json(v.iter().map(|s| (*s).into()).collect());
        }

        if let Some(v) = value.salience {
            row.salience = v;
        }

        if let Some(v) = value.strength {
            row.strength = v;
        }

        if let Some(v) = value.confidence {
            row.confidence = v;
        }

        if let Some(v) = value.recalls {
            row.recalls = v as i64;
        }

        if let Some(v) = value.recalled_at {
            row.recalled_at = Some(v);
        }

        let row = self.storage.memories().update(row).await?;
        let artifacts = self.storage.artifacts().find_by_memory_id(id.into()).await?;

        Ok(Memory {
            id: row.id.into(),
            version: row.version.into(),
            ty: row.ty.into(),
            contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
            sources: row.sources.iter().map(|v| (*v).into()).collect(),
            artifacts: artifacts.iter().map(|v| v.clone().into()).collect(),
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
