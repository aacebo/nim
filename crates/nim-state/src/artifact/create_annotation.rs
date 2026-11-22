use nim_store::ArtifactAnnotationRow;

use crate::{ArtifactAnnotation, ArtifactId, ArtifactService, Label, Span};

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
    ) -> nim_error::Result<ArtifactAnnotation> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .artifact_annotations()
            .create(ArtifactAnnotationRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                artifact_id: artifact_id.into(),
                label: value.label.to_string(),
                text: value.text,
                spans: serde_json::to_value(value.spans)?,
                confidence: value.confidence,
                embedding: value.embedding.map(|v| v.to_vec().into()),
                created_at: now,
                updated_at: now,
            })
            .await?;

        row.try_into()
    }
}
