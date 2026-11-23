use crate::{MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: MessageId) -> nim_error::Result<()> {
        let message = self.storage.messages().find_by_id(id.into()).await?;
        self.storage.messages().delete(id.into()).await?;
        let mut chat = self.storage.chats().find_by_id(message.chat_id).await?;
        let last_message = self.find_last_message(message.chat_id.into()).await?;

        chat.last_message_id = last_message.as_ref().map(|message| message.id.into());
        chat.last_message_at = last_message.map(|message| message.created_at);
        self.storage.chats().update(chat).await?;
        Ok(())
    }
}
