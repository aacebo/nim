use crate::{Message, MessageId, MessageService, Page, PageResponse};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_reply_to_id(&self, reply_to_id: MessageId, page: Page) -> nim_error::Result<PageResponse<Message>> {
        let rows = self.storage.messages().find_by_reply_to_id(reply_to_id.into(), page).await?;
        rows.try_into()
    }
}
