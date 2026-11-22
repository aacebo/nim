use crate::{AnnotationId, Label, MessageAnnotation, MessageId, MessageService, Span};

#[derive(Debug, Default)]
pub struct MessageAnnotationUpdate {
    pub label: Option<Label>,
    pub text: Option<String>,
    pub spans: Option<Vec<Span>>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
}

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update_annotation(
        &self,
        message_id: MessageId,
        id: AnnotationId,
        value: MessageAnnotationUpdate,
    ) -> nim_error::Result<MessageAnnotation> {
        let mut row = self
            .storage
            .message_annotations()
            .find_by_id(message_id.into(), id.into())
            .await?;

        if let Some(value) = value.label {
            row.label = value.to_string();
        }

        if let Some(value) = value.text {
            row.text = value;
        }

        if let Some(value) = value.spans {
            row.spans = serde_json::to_value(value)?;
        }

        if let Some(value) = value.confidence {
            row.confidence = value;
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        self.storage.message_annotations().update(row).await?.try_into()
    }
}
