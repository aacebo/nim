use crate::types;

pub struct PredictionStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> PredictionStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::PredictionRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                hypothesis,
                confidence,
                deadline,
                created_at,
                updated_at
            FROM predictions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::PredictionRow) -> Result<types::PredictionRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO predictions (
                id,
                version,
                hypothesis,
                confidence,
                deadline,
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
                $7
            )
            RETURNING
                id,
                version,
                hypothesis,
                confidence,
                deadline,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.version)
        .bind(value.hypothesis)
        .bind(value.confidence)
        .bind(value.deadline)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::PredictionRow) -> Result<types::PredictionRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE predictions
            SET
                version = version + 1,
                hypothesis = $2,
                confidence = $3,
                deadline = $4,
                updated_at = $5
            WHERE id = $1
            RETURNING
                id,
                version,
                hypothesis,
                confidence,
                deadline,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.hypothesis)
        .bind(value.confidence)
        .bind(value.deadline)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM predictions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

pub struct PredictionMemoriesStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> PredictionMemoriesStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        prediction_id: uuid::Uuid,
        memory_id: uuid::Uuid,
    ) -> Result<types::PredictionMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                prediction_id,
                memory_id,
                created_at
            FROM predictions_memories
            WHERE prediction_id = $1 AND memory_id = $2
            "#,
        )
        .bind(prediction_id)
        .bind(memory_id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::PredictionMemoryRow) -> Result<types::PredictionMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO predictions_memories (
                prediction_id,
                memory_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING
                prediction_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.prediction_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::PredictionMemoryRow) -> Result<types::PredictionMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE predictions_memories
            SET
                created_at = $3
            WHERE prediction_id = $1 AND memory_id = $2
            RETURNING
                prediction_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.prediction_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM predictions_memories
            WHERE prediction_id = $1 AND memory_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
