use crate::{Chat, ChatService, Page, PageResponse};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find(&self, page: Page) -> nim_error::Result<PageResponse<Chat>> {
        let rows = self.storage.chats().find(page).await?;
        Ok(rows.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_latest(&self) -> nim_error::Result<Option<Chat>> {
        Ok(self.storage.chats().find_latest().await?.map(Into::into))
    }
}
