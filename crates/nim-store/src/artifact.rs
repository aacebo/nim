use crate::types;

#[derive(Copy, Clone)]
pub struct ArtifactStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ArtifactStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::ArtifactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                name,
                content,
                created_at,
                updated_at
            FROM artifacts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_memory_id(&self, memory_id: uuid::Uuid) -> Result<Vec<types::ArtifactRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                artifacts.id,
                artifacts.name,
                artifacts.content,
                artifacts.created_at,
                artifacts.updated_at
            FROM artifacts
            JOIN memories_artifacts ON memories_artifacts.artifact_id = artifacts.id
            WHERE memories_artifacts.memory_id = $1
            ORDER BY artifacts.created_at, artifacts.id
            "#,
        )
        .bind(memory_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self, value))]
    pub async fn create(&self, value: types::ArtifactRow) -> Result<types::ArtifactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO artifacts (
                id,
                type,
                name,
                content,
                created_at,
                updated_at
            )
            VALUES (
                $1,
                $3::jsonb ->> 'type',
                $2,
                $3,
                $4,
                $5
            )
            RETURNING
                id,
                name,
                content,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.name)
        .bind(value.content)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self, value))]
    pub async fn update(&self, value: types::ArtifactRow) -> Result<types::ArtifactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE artifacts
            SET
                type = $3::jsonb ->> 'type',
                name = $2,
                content = $3,
                updated_at = $4
            WHERE id = $1
            RETURNING
                id,
                name,
                content,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.name)
        .bind(value.content)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn link(&self, memory_id: uuid::Uuid, artifact_id: uuid::Uuid) -> Result<types::MemoryArtifactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO memories_artifacts (
                memory_id,
                artifact_id
            )
            VALUES (
                $1,
                $2
            )
            RETURNING
                memory_id,
                artifact_id,
                created_at
            "#,
        )
        .bind(memory_id)
        .bind(artifact_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn unlink(&self, memory_id: uuid::Uuid, artifact_id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM memories_artifacts
            WHERE memory_id = $1 AND artifact_id = $2
            "#,
        )
        .bind(memory_id)
        .bind(artifact_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM artifacts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
