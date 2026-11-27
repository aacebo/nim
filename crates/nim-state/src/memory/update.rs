use crate::{Embedding, Memory, MemoryId, MemoryService, Source};

#[derive(Debug, Default)]
pub struct MemoryUpdate<'a> {
    pub sources: Option<&'a [Source]>,
    pub summary: Option<String>,
    pub embedding: Option<Embedding>,
    pub salience: Option<f32>,
    pub strength: Option<f32>,
    pub confidence: Option<f32>,
    pub recalls: Option<u64>,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update<'b>(&self, id: MemoryId, value: MemoryUpdate<'b>) -> nim_error::Result<Memory> {
        let mut row = self.storage.memories().find_by_id(id.into()).await?;

        if let Some(v) = value.sources {
            row.sources = serde_json::to_value(v)?;
        }

        if let Some(v) = value.summary {
            row.summary = Some(v);
        }

        if let Some(v) = value.embedding {
            row.embedding = Some(v.into());
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

        self.storage.memories().update(row).await?.try_into()
    }
}
