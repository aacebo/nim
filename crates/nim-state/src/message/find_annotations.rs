use crate::{Error, MessageAnnotation, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotations(&self, message_id: MessageId) -> Result<Vec<MessageAnnotation>, Error> {
        Ok(self
            .storage
            .message_annotations()
            .find_by_message_id(message_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
