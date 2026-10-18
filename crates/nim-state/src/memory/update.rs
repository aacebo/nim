use crate::{Error, Memory, MemoryId, MemoryService};

#[derive(Debug, Default)]
pub struct MemoryUpdate {
    pub salience: Option<f32>,
    pub strength: Option<f32>,
    pub confidence: Option<f32>,
    pub recalls: Option<u64>,
    pub summary: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: MemoryId, value: MemoryUpdate) -> Result<Memory, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

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

        if let Some(v) = value.summary {
            row.summary = Some(v);
        }

        if let Some(v) = value.embedding {
            row.embedding = Some(v.into());
        }

        if let Some(v) = value.recalled_at {
            row.recalled_at = Some(v);
        }

        let row = self.store.update(row).await?;

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
