use crate::{Memory, MemoryService, Page, PageResponse};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find(&self, page: Page) -> nim_error::Result<PageResponse<Memory>> {
        let rows = self.storage.memories().find(page).await?;
        let ids: Vec<_> = rows.iter().map(|row| row.id).collect();
        let relations = self.storage.memory_relations().find_by_memory_ids(&ids).await?;
        let mut memories = Vec::with_capacity(rows.len());

        for row in rows.data {
            let id = row.id;
            let artifacts = self.storage.artifacts().find_by_memory_id(id).await?;
            let mut memory: Memory = row.try_into()?;

            memory.artifacts = artifacts
                .into_iter()
                .map(TryInto::try_into)
                .collect::<nim_error::Result<_>>()?;
            memory.relations = relations
                .iter()
                .filter_map(|relation| {
                    if relation.source_id == id {
                        Some(relation.target_id.into())
                    } else if relation.target_id == id {
                        Some(relation.source_id.into())
                    } else {
                        None
                    }
                })
                .collect();
            memories.push(memory);
        }

        Ok(PageResponse {
            size: rows.size,
            next: rows.next,
            data: memories,
        })
    }
}
