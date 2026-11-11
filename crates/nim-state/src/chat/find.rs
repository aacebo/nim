use crate::{Chat, ChatService, Error, Page, PageResponse};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find(&self, page: Page) -> Result<PageResponse<Chat>, Error> {
        let rows = self.storage.chats().find(page).await?;
        Ok(rows.into())
    }
}
