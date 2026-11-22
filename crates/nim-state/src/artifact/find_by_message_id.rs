use crate::{Artifact, ArtifactService, MessageId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_message_id(&self, message_id: MessageId) -> nim_error::Result<Vec<Artifact>> {
        self.storage
            .artifacts()
            .find_by_message_id(message_id.into())
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }
}
