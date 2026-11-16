use crate::{Label, MessageAnnotation, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotation_by_label(
        &self,
        message_id: MessageId,
        label: Label,
        text: &str,
    ) -> nim_error::Result<Option<MessageAnnotation>> {
        Ok(self
            .storage
            .message_annotations()
            .find_by_message_label(message_id.into(), label.into(), text)
            .await?
            .map(Into::into))
    }
}
