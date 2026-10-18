use crate::{Annotation, AnnotationId, AnnotationService, Error, Label, MemoryId, Span};

#[derive(Debug, Default)]
pub struct AnnotationUpdate {
    pub memory_id: Option<MemoryId>,
    pub label: Option<Label>,
    pub text: Option<String>,
    pub spans: Option<Vec<Span>>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
}

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: AnnotationId, value: AnnotationUpdate) -> Result<Annotation, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.memory_id {
            row.memory_id = value.into();
        }

        if let Some(value) = value.label {
            row.label = value.into();
        }

        if let Some(value) = value.text {
            row.text = value;
        }

        if let Some(value) = value.spans {
            row.spans = nim_store::Json(value.into_iter().map(Into::into).collect());
        }

        if let Some(value) = value.confidence {
            row.confidence = value as f64;
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        Ok(self.store.update(row).await?.into())
    }
}
