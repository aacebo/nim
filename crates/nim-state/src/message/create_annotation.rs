use nim_store::MessageAnnotationRow;

use crate::{Error, Label, MessageAnnotation, MessageId, MessageService, Span};

#[derive(Debug)]
pub struct MessageAnnotationCreate<'a> {
    pub label: Label,
    pub text: String,
    pub spans: &'a [Span],
    pub confidence: f32,
    pub embedding: Option<&'a [f32]>,
}

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create_annotation<'b>(
        &self,
        message_id: MessageId,
        value: MessageAnnotationCreate<'b>,
    ) -> Result<MessageAnnotation, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .message_annotations()
            .create(MessageAnnotationRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                message_id: message_id.into(),
                label: value.label.into(),
                text: value.text,
                spans: nim_store::Json(value.spans.to_vec()),
                confidence: value.confidence,
                embedding: value.embedding.map(|v| v.to_vec().into()),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
