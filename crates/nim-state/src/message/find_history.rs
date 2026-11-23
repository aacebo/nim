use crate::{ChatId, Message, MessageId, MessageService, PageResponse};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_history(&self, chat_id: ChatId, id: MessageId, count: usize) -> nim_error::Result<PageResponse<Message>> {
        let rows = self.storage.messages().find_history(chat_id.into(), id.into(), count).await?;
        rows.try_into()
    }
}
