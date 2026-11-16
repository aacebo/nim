use crate::{ChatId, ChatService};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: ChatId) -> nim_error::Result<()> {
        Ok(self.storage.chats().delete(id.into()).await?)
    }
}
