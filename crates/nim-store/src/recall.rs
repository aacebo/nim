use crate::types;

#[derive(Copy, Clone)]
pub struct RecallStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> RecallStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::RecallRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                trigger,
                source,
                query,
                created_at
            FROM recalls
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::RecallRow) -> Result<types::RecallRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO recalls (
                id,
                trigger,
                source,
                query,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            )
            RETURNING
                id,
                trigger,
                source,
                query,
                created_at
            "#,
        )
        .bind(value.id)
        .bind(value.trigger)
        .bind(value.source)
        .bind(value.query)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::RecallRow) -> Result<types::RecallRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE recalls
            SET
                trigger = $2,
                source = $3,
                query = $4
            WHERE id = $1
            RETURNING
                id,
                trigger,
                source,
                query,
                created_at
            "#,
        )
        .bind(value.id)
        .bind(value.trigger)
        .bind(value.source)
        .bind(value.query)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM recalls
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
pub struct MemoryRecallsStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MemoryRecallsStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, memory_id: uuid::Uuid, recall_id: uuid::Uuid) -> Result<types::MemoryRecallRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                memory_id,
                recall_id,
                score,
                created_at
            FROM memories_recalls
            WHERE memory_id = $1 AND recall_id = $2
            "#,
        )
        .bind(memory_id)
        .bind(recall_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MemoryRecallRow) -> Result<types::MemoryRecallRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO memories_recalls (
                memory_id,
                recall_id,
                score,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING
                memory_id,
                recall_id,
                score,
                created_at
            "#,
        )
        .bind(value.memory_id)
        .bind(value.recall_id)
        .bind(value.score)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::MemoryRecallRow) -> Result<types::MemoryRecallRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE memories_recalls
            SET
                score = $3,
                created_at = $4
            WHERE memory_id = $1 AND recall_id = $2
            RETURNING
                memory_id,
                recall_id,
                score,
                created_at
            "#,
        )
        .bind(value.memory_id)
        .bind(value.recall_id)
        .bind(value.score)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM memories_recalls
            WHERE memory_id = $1 AND recall_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
