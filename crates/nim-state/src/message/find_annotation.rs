use crate::{AnnotationId, Error, MessageAnnotation, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotation(&self, message_id: MessageId, id: AnnotationId) -> Result<MessageAnnotation, Error> {
        Ok(self
            .storage
            .message_annotations()
            .find_by_id(message_id.into(), id.into())
            .await?
            .into())
    }
}
