use crate::{ArtifactId, Chat, ChatService, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: ArtifactId) -> Result<Vec<Chat>, Error> {
        let rows = self.storage.chats().find_by_artifact_id(artifact_id.into()).await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
