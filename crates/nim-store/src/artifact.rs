use crate::{Cursor, Page, PageResponse, types};

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
    pub async fn find_by_chat_id(
        &self,
        chat_id: uuid::Uuid,
        page: Page,
    ) -> Result<PageResponse<types::ArtifactRow>, sqlx::Error> {
        let (cursor_id, cursor_at) = page.cursor.map(|cursor| (cursor.id, cursor.timestamp)).unzip();
        let data: Vec<types::ArtifactRow> = sqlx::query_as(
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
            JOIN chats_artifacts ON chats_artifacts.artifact_id = artifacts.id
            WHERE chats_artifacts.chat_id = $1
            AND ($2::timestamptz IS NULL OR (artifacts.created_at, artifacts.id) > ($2, $3::uuid))
            ORDER BY artifacts.created_at, artifacts.id
            LIMIT $4
            "#,
        )
        .bind(chat_id)
        .bind(cursor_at)
        .bind(cursor_id)
        .bind(i64::try_from(page.limit).unwrap_or(i64::MAX))
        .fetch_all(self.pool)
        .await?;

        Ok(PageResponse {
            size: page.limit,
            next: data.last().filter(|_| data.len() == page.limit).map(|v| Cursor {
                id: v.id,
                timestamp: v.created_at,
            }),
            data,
        })
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
                confidence = $6,
                salience = GREATEST(salience, $7),
                strength = $8,
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

#[derive(Copy, Clone)]
pub struct ArtifactAnnotationStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ArtifactAnnotationStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, artifact_id: uuid::Uuid, id: uuid::Uuid) -> Result<types::ArtifactAnnotationRow, sqlx::Error> {
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
            FROM artifact_annotations
            WHERE id = $1 AND artifact_id = $2
            "#,
        )
        .bind(id)
        .bind(artifact_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: uuid::Uuid) -> Result<Vec<types::ArtifactAnnotationRow>, sqlx::Error> {
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
            FROM artifact_annotations
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
    ) -> Result<Option<types::ArtifactAnnotationRow>, sqlx::Error> {
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
            FROM artifact_annotations
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
    pub async fn create(&self, value: types::ArtifactAnnotationRow) -> Result<types::ArtifactAnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO artifact_annotations (
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
    pub async fn update(&self, value: types::ArtifactAnnotationRow) -> Result<types::ArtifactAnnotationRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE artifact_annotations
            SET
                version = version + 1,
                label = $2,
                text = $3,
                spans = $4,
                confidence = $5,
                embedding = $6,
                updated_at = $7
            WHERE id = $1 AND version = $8 AND artifact_id = $9
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
        .bind(value.artifact_id)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.artifact_id, value.id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, artifact_id: uuid::Uuid, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM artifact_annotations
            WHERE id = $1 AND artifact_id = $2
            "#,
        )
        .bind(id)
        .bind(artifact_id)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}
