use nim_store::AnnotationRow;

use crate::{Annotation, AnnotationService, Error, Label, MemoryId, Span};

#[derive(Debug)]
pub struct AnnotationCreate<'a> {
    pub memory_id: MemoryId,
    pub label: Label,
    pub text: String,
    pub spans: &'a [Span],
    pub confidence: f32,
    pub embedding: Option<&'a [f32]>,
}

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create<'b>(&self, value: AnnotationCreate<'b>) -> Result<Annotation, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(AnnotationRow {
                id: uuid::Uuid::now_v7(),
                memory_id: value.memory_id.into(),
                label: value.label.into(),
                text: value.text,
                spans: nim_store::Json(value.spans.iter().copied().map(Into::into).collect()),
                confidence: value.confidence,
                embedding: value.embedding.map(|v| v.to_vec().into()),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
