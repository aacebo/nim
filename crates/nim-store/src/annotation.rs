use crate::types;

#[derive(Copy, Clone)]
pub struct AnnotationStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> AnnotationStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::AnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                artifact_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM annotations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: uuid::Uuid) -> Result<Vec<types::AnnotationRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                artifact_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM annotations
            WHERE artifact_id = $1
            "#,
        )
        .bind(artifact_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_label(
        &self,
        artifact_id: uuid::Uuid,
        label: types::Label,
        text: &str,
    ) -> Result<Option<types::AnnotationRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                artifact_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM annotations
            WHERE artifact_id = $1
            AND label = $2
            AND text = $3
            "#,
        )
        .bind(artifact_id)
        .bind(label)
        .bind(text)
        .fetch_optional(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::AnnotationRow) -> Result<types::AnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO annotations (
                id,
                artifact_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at,
                version
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10
            )
            RETURNING
                id,
                version,
                artifact_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.artifact_id)
        .bind(value.label)
        .bind(value.text)
        .bind(value.spans)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.created_at)
        .bind(value.updated_at)
        .bind(value.version)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::AnnotationRow) -> Result<types::AnnotationRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE annotations
            SET
                version = version + 1,
                label = $2,
                text = $3,
                spans = $4,
                confidence = GREATEST(confidence, $5),
                embedding = $6,
                updated_at = $7
            WHERE id = $1 AND version = $8
            RETURNING
                id,
                version,
                artifact_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.label)
        .bind(value.text)
        .bind(value.spans)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(chrono::Utc::now())
        .bind(value.version)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM annotations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
