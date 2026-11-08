use crate::{Artifact, ArtifactService, ChatId, Error, Page};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId, page: Page) -> Result<Vec<Artifact>, Error> {
        let rows = self.storage.artifacts().find_by_chat_id(chat_id.into(), page).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
