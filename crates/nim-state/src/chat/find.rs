use crate::{Chat, ChatService, Error, Page};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find(&self, page: Page) -> Result<Vec<Chat>, Error> {
        let rows = self.storage.chats().find(page).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
