use crate::{Artifact, ArtifactService, ChatId, Error, Page, PageResponse};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId, page: Page) -> Result<PageResponse<Artifact>, Error> {
        let rows = self.storage.artifacts().find_by_chat_id(chat_id.into(), page).await?;
        Ok(rows.into())
    }
}
