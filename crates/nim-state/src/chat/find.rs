use crate::{Chat, ChatService, Page, PageResponse};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find(&self, page: Page) -> nim_error::Result<PageResponse<Chat>> {
        let rows = self.storage.chats().find(page).await?;
        Ok(rows.into())
    }
}
