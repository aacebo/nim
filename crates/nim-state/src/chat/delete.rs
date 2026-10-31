use crate::{ChatId, ChatService, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: ChatId) -> Result<(), Error> {
        Ok(self.storage.chats().delete(id.into()).await?)
    }
}
