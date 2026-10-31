use crate::{ArtifactId, ChatArtifact, ChatEntity, ChatId, ChatService, EntityId, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link_entity(&self, chat_id: ChatId, entity_id: EntityId) -> Result<ChatEntity, Error> {
        let row = self
            .storage
            .chat_entities()
            .create(nim_store::ChatEntityRow {
                chat_id: chat_id.into(),
                entity_id: entity_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn link_artifact(&self, chat_id: ChatId, artifact_id: ArtifactId) -> Result<ChatArtifact, Error> {
        let row = self
            .storage
            .chat_artifacts()
            .create(nim_store::ChatArtifactRow {
                chat_id: chat_id.into(),
                artifact_id: artifact_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
