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
                version,
                name,
                content,
                summary,
                embedding,
                confidence,
                salience,
                strength,
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
                artifacts.version,
                artifacts.name,
                artifacts.content,
                artifacts.summary,
                artifacts.embedding,
                artifacts.confidence,
                artifacts.salience,
                artifacts.strength,
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
                summary,
                embedding,
                confidence,
                salience,
                strength,
                created_at,
                updated_at,
                version
            )
            VALUES (
                $1,
                $3::jsonb ->> 'type',
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11
            )
            RETURNING
                id,
                version,
                name,
                content,
                summary,
                embedding,
                confidence,
                salience,
                strength,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.name)
        .bind(value.content)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.confidence)
        .bind(value.salience)
        .bind(value.strength)
        .bind(value.created_at)
        .bind(value.updated_at)
        .bind(value.version)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self, value))]
    pub async fn update(&self, value: types::ArtifactRow) -> Result<types::ArtifactRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE artifacts
            SET
                version = version + 1,
                type = $3::jsonb ->> 'type',
                name = $2,
                content = $3,
                summary = $4,
                embedding = $5,
                confidence = GREATEST(confidence, $6),
                salience = GREATEST(salience, $7),
                strength = GREATEST(strength, $8),
                updated_at = $9
            WHERE id = $1 AND version = $10
            RETURNING
                id,
                version,
                name,
                content,
                summary,
                embedding,
                confidence,
                salience,
                strength,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.name)
        .bind(value.content)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.confidence)
        .bind(value.salience)
        .bind(value.strength)
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

#[derive(Copy, Clone)]
pub struct MemoryArtifactsStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MemoryArtifactsStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(
        &self,
        memory_id: uuid::Uuid,
        artifact_id: uuid::Uuid,
    ) -> Result<types::MemoryArtifactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                memory_id,
                artifact_id,
                created_at
            FROM memories_artifacts
            WHERE memory_id = $1 AND artifact_id = $2
            "#,
        )
        .bind(memory_id)
        .bind(artifact_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MemoryArtifactRow) -> Result<types::MemoryArtifactRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO memories_artifacts (
                memory_id,
                artifact_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            ON CONFLICT DO NOTHING
            RETURNING
                memory_id,
                artifact_id,
                created_at
            "#,
        )
        .bind(value.memory_id)
        .bind(value.artifact_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.memory_id, value.artifact_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM memories_artifacts
            WHERE memory_id = $1 AND artifact_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
