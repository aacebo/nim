use crate::{Chat, ChatService, EntityId};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_entity_id(&self, entity_id: EntityId) -> nim_error::Result<Vec<Chat>> {
        let rows = self.storage.chats().find_by_entity_id(entity_id.into()).await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
