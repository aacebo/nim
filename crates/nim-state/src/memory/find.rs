use crate::{Error, Memory, MemoryService, Page, PageResponse};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find(&self, page: Page) -> Result<PageResponse<Memory>, Error> {
        let rows = self.storage.memories().find(page).await?;
        let ids: Vec<_> = rows.iter().map(|row| row.id).collect();
        let relations = self.storage.memory_relations().find_by_memory_ids(&ids).await?;
        let mut memories = Vec::with_capacity(rows.len());

        for row in rows.data {
            let artifacts = self.storage.artifacts().find_by_memory_id(row.id).await?;

            memories.push(Memory {
                id: row.id.into(),
                version: row.version.into(),
                ty: row.ty.into(),
                contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
                sources: row.sources.iter().map(|v| (*v).into()).collect(),
                artifacts: artifacts.iter().map(|v| v.clone().into()).collect(),
                relations: relations
                    .iter()
                    .filter_map(|relation| {
                        if relation.source_id == row.id {
                            Some(relation.target_id.into())
                        } else if relation.target_id == row.id {
                            Some(relation.source_id.into())
                        } else {
                            None
                        }
                    })
                    .collect(),
                summary: row.summary,
                embedding: row.embedding.map(Into::into),
                salience: row.salience,
                strength: row.strength,
                confidence: row.confidence,
                recalls: row.recalls as u64,
                recalled_at: row.recalled_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(PageResponse {
            size: rows.size,
            next: rows.next,
            data: memories,
        })
    }
}
