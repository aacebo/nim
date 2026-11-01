use crate::{AnnotationId, Error, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete_annotation(&self, message_id: MessageId, id: AnnotationId) -> Result<(), Error> {
        Ok(self
            .storage
            .message_annotations()
            .delete(message_id.into(), id.into())
            .await?)
    }
}
