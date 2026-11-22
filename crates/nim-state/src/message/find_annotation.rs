use crate::{AnnotationId, MessageAnnotation, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotation(&self, message_id: MessageId, id: AnnotationId) -> nim_error::Result<MessageAnnotation> {
        self.storage
            .message_annotations()
            .find_by_id(message_id.into(), id.into())
            .await?
            .try_into()
    }
}
