use crate::{AnnotationId, ArtifactAnnotation, ArtifactId, ArtifactService, Error, Label, Span};

#[derive(Debug, Default)]
pub struct ArtifactAnnotationUpdate {
    pub label: Option<Label>,
    pub text: Option<String>,
    pub spans: Option<Vec<Span>>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update_annotation(
        &self,
        artifact_id: ArtifactId,
        id: AnnotationId,
        value: ArtifactAnnotationUpdate,
    ) -> Result<ArtifactAnnotation, Error> {
        let mut row = self
            .storage
            .artifact_annotations()
            .find_by_id(artifact_id.into(), id.into())
            .await?;

        if let Some(value) = value.label {
            row.label = value.into();
        }

        if let Some(value) = value.text {
            row.text = value;
        }

        if let Some(value) = value.spans {
            row.spans = nim_store::Json(value);
        }

        if let Some(value) = value.confidence {
            row.confidence = value;
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        Ok(self.storage.artifact_annotations().update(row).await?.into())
    }
}
