use crate::types;

#[derive(Copy, Clone)]
pub struct ChatStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ChatStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::ChatRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                summary,
                embedding,
                confidence,
                last_analyzed_id,
                last_analyzed_at,
                created_at,
                updated_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_entity_id(&self, entity_id: uuid::Uuid) -> Result<Vec<types::ChatRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT chats.*
            FROM chats
            JOIN chats_entities ON chats_entities.chat_id = chats.id
            WHERE chats_entities.entity_id = $1
            "#,
        )
        .bind(entity_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: uuid::Uuid) -> Result<Vec<types::ChatRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT chats.*
            FROM chats
            JOIN chats_artifacts ON chats_artifacts.chat_id = chats.id
            WHERE chats_artifacts.artifact_id = $1
            "#,
        )
        .bind(artifact_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ChatRow) -> Result<types::ChatRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO chats (
                id,
                version,
                summary,
                embedding,
                confidence,
                last_analyzed_id,
                last_analyzed_at,
                created_at,
                updated_at
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
                $9
            )
            RETURNING
                id,
                version,
                summary,
                embedding,
                confidence,
                last_analyzed_id,
                last_analyzed_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.version)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.confidence)
        .bind(value.last_analyzed_id)
        .bind(value.last_analyzed_at)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::ChatRow) -> Result<types::ChatRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE chats
            SET
                version = version + 1,
                summary = $2,
                embedding = $3,
                confidence = $4,
                last_analyzed_id = $5,
                last_analyzed_at = $6,
                updated_at = $7
            WHERE id = $1 AND version = $8
            RETURNING
                id,
                version,
                summary,
                embedding,
                confidence,
                last_analyzed_id,
                last_analyzed_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.confidence)
        .bind(value.last_analyzed_id)
        .bind(value.last_analyzed_at)
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
            DELETE FROM chats
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
pub struct ChatEntitiesStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ChatEntitiesStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, chat_id: uuid::Uuid, entity_id: uuid::Uuid) -> Result<types::ChatEntityRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                chat_id,
                entity_id,
                created_at
            FROM chats_entities
            WHERE chat_id = $1 AND entity_id = $2
            "#,
        )
        .bind(chat_id)
        .bind(entity_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ChatEntityRow) -> Result<types::ChatEntityRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO chats_entities (
                chat_id,
                entity_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            ON CONFLICT DO NOTHING
            RETURNING
                chat_id,
                entity_id,
                created_at
            "#,
        )
        .bind(value.chat_id)
        .bind(value.entity_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.chat_id, value.entity_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM chats_entities
            WHERE chat_id = $1 AND entity_id = $2
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
pub struct ChatArtifactsStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ChatArtifactsStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, chat_id: uuid::Uuid, artifact_id: uuid::Uuid) -> Result<types::ChatArtifactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                chat_id,
                artifact_id,
                created_at
            FROM chats_artifacts
            WHERE chat_id = $1 AND artifact_id = $2
            "#,
        )
        .bind(chat_id)
        .bind(artifact_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ChatArtifactRow) -> Result<types::ChatArtifactRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO chats_artifacts (
                chat_id,
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
                chat_id,
                artifact_id,
                created_at
            "#,
        )
        .bind(value.chat_id)
        .bind(value.artifact_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.chat_id, value.artifact_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM chats_artifacts
            WHERE chat_id = $1 AND artifact_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
