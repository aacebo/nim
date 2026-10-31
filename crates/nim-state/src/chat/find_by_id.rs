use crate::{Chat, ChatId, ChatService, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: ChatId) -> Result<Chat, Error> {
        Ok(self.storage.chats().find_by_id(id.into()).await?.into())
    }
}
