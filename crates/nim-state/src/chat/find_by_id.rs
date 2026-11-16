use crate::{Chat, ChatId, ChatService};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: ChatId) -> nim_error::Result<Chat> {
        Ok(self.storage.chats().find_by_id(id.into()).await?.into())
    }
}
