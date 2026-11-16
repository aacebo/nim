use crate::{AnnotationId, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete_annotation(&self, message_id: MessageId, id: AnnotationId) -> nim_error::Result<()> {
        Ok(self
            .storage
            .message_annotations()
            .delete(message_id.into(), id.into())
            .await?)
    }
}
