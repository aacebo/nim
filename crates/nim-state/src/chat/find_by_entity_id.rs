use crate::{Chat, ChatService, EntityId, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_entity_id(&self, entity_id: EntityId) -> Result<Vec<Chat>, Error> {
        let rows = self.storage.chats().find_by_entity_id(entity_id.into()).await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
