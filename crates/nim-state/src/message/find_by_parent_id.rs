use crate::{Message, MessageId, MessageService, Page, PageResponse};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_parent_id(&self, parent_id: MessageId, page: Page) -> nim_error::Result<PageResponse<Message>> {
        let rows = self.storage.messages().find_by_parent_id(parent_id.into(), page).await?;
        rows.try_into()
    }
}
