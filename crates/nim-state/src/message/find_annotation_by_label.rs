use crate::{Error, Label, MessageAnnotation, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotation_by_label(
        &self,
        message_id: MessageId,
        label: Label,
        text: &str,
    ) -> Result<Option<MessageAnnotation>, Error> {
        Ok(self
            .storage
            .message_annotations()
            .find_by_message_label(message_id.into(), label.into(), text)
            .await?
            .map(Into::into))
    }
}
