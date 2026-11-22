use crate::{MessageAnnotation, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotations(&self, message_id: MessageId) -> nim_error::Result<Vec<MessageAnnotation>> {
        self.storage
            .message_annotations()
            .find_by_message_id(message_id.into())
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }
}
