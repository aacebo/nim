use nim_store::ArtifactRow;

use crate::{Artifact, ArtifactContent, ArtifactService, Embedding, Error};

#[derive(Debug)]
pub struct ArtifactCreate {
    pub name: String,
    pub content: ArtifactContent,
    pub summary: Option<String>,
    pub embedding: Option<Embedding>,
    pub confidence: Option<f32>,
    pub salience: Option<f32>,
    pub strength: Option<f32>,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self, value))]
    pub async fn create(&self, value: ArtifactCreate) -> Result<Artifact, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .artifacts()
            .create(ArtifactRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                name: value.name,
                content: nim_store::Json(value.content),
                summary: value.summary,
                embedding: value.embedding.map(Into::into),
                confidence: value.confidence.unwrap_or_default(),
                salience: value.salience.unwrap_or_default(),
                strength: value.strength.unwrap_or_default(),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
