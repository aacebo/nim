use crate::{ChatId, Message, MessageService, Page, PageResponse};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId, page: Page) -> nim_error::Result<PageResponse<Message>> {
        let rows = self.storage.messages().find_by_chat_id(chat_id.into(), page).await?;
        rows.try_into()
    }
}
