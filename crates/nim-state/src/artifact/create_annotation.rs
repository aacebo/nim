use nim_store::ArtifactAnnotationRow;

use crate::{ArtifactAnnotation, ArtifactId, ArtifactService, Error, Label, Span};

#[derive(Debug)]
pub struct ArtifactAnnotationCreate<'a> {
    pub label: Label,
    pub text: String,
    pub spans: &'a [Span],
    pub confidence: f32,
    pub embedding: Option<&'a [f32]>,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create_annotation<'b>(
        &self,
        artifact_id: ArtifactId,
        value: ArtifactAnnotationCreate<'b>,
    ) -> Result<ArtifactAnnotation, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .artifact_annotations()
            .create(ArtifactAnnotationRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                artifact_id: artifact_id.into(),
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
