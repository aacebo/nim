use crate::{ArtifactId, Error, Memory, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: ArtifactId) -> Result<Vec<Memory>, Error> {
        let rows = self.storage.memories().find_by_artifact_id(artifact_id.into()).await?;
        let mut memories = Vec::with_capacity(rows.len());

        for row in rows {
            let artifacts = self.storage.artifacts().find_by_memory_id(row.id).await?;

            memories.push(Memory {
                id: row.id.into(),
                version: Default::default(),
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
            });
        }

        Ok(memories)
    }
}
