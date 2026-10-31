use crate::{ArtifactId, ChatId, ChatService, EntityId, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink_entity(&self, chat_id: ChatId, entity_id: EntityId) -> Result<(), Error> {
        self.storage
            .chat_entities()
            .delete((chat_id.into(), entity_id.into()))
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn unlink_artifact(&self, chat_id: ChatId, artifact_id: ArtifactId) -> Result<(), Error> {
        self.storage
            .chat_artifacts()
            .delete((chat_id.into(), artifact_id.into()))
            .await?;

        Ok(())
    }
}
