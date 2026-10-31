use crate::{MessageFilter, SearchItem, types};

#[derive(Copy, Clone)]
pub struct MessageStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MessageStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::MessageRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                chat_id,
                version,
                role,
                content,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM messages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: uuid::Uuid) -> Result<Vec<types::MessageRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                chat_id,
                version,
                role,
                content,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM messages
            WHERE chat_id = $1
            ORDER BY created_at, id
            "#,
        )
        .bind(chat_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn search(
        &self,
        embedding: &[f32],
        filter: MessageFilter,
    ) -> Result<Vec<SearchItem<types::MessageRow>>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                (1 - (embedding <=> $1::vector))::real AS similarity,
                messages.*
            FROM messages
            WHERE embedding IS NOT NULL
            AND (1 - (embedding <=> $1::vector))::real >= $2
            AND ($4::uuid IS NULL OR chat_id = $4)
            AND ($5::text IS NULL OR role = $5)
            ORDER BY embedding <=> $1::vector, id
            LIMIT $3
            "#,
        )
        .bind(pgvector::Vector::from(embedding.to_vec()))
        .bind(filter.min_score.unwrap_or(0.2))
        .bind(filter.limit.map(|value| value as i64).unwrap_or(10))
        .bind(filter.chat_id)
        .bind(filter.role)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MessageRow) -> Result<types::MessageRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO messages (
                id,
                chat_id,
                version,
                role,
                content,
                summary,
                confidence,
                embedding,
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
                $9,
                $10
            )
            RETURNING
                id,
                chat_id,
                version,
                role,
                content,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.chat_id)
        .bind(value.version)
        .bind(value.role)
        .bind(value.content)
        .bind(value.summary)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::MessageRow) -> Result<types::MessageRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE messages
            SET
                version = version + 1,
                content = $2,
                summary = $3,
                confidence = $4,
                embedding = $5,
                updated_at = $6
            WHERE id = $1 AND version = $7
            RETURNING
                id,
                chat_id,
                version,
                role,
                content,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.content)
        .bind(value.summary)
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
            DELETE FROM messages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
