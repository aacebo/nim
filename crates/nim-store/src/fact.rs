use crate::types;

pub struct FactStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> FactStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::FactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                description,
                confidence,
                embedding,
                recalls,
                recalled_at,
                created_at,
                updated_at
            FROM facts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::FactRow) -> Result<types::FactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO facts (
                id,
                description,
                confidence,
                embedding,
                recalls,
                recalled_at,
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
                $8
            )
            RETURNING
                id,
                description,
                confidence,
                embedding,
                recalls,
                recalled_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.description)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.recalls)
        .bind(value.recalled_at)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::FactRow) -> Result<types::FactRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE facts
            SET
                description = $2,
                confidence = $3,
                embedding = $4,
                recalls = $5,
                recalled_at = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING
                id,
                description,
                confidence,
                embedding,
                recalls,
                recalled_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.description)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.recalls)
        .bind(value.recalled_at)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM facts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

pub struct FactMemoriesStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> FactMemoriesStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, fact_id: uuid::Uuid, memory_id: uuid::Uuid) -> Result<types::FactMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                fact_id,
                memory_id,
                created_at
            FROM facts_memories
            WHERE fact_id = $1 AND memory_id = $2
            "#,
        )
        .bind(fact_id)
        .bind(memory_id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::FactMemoryRow) -> Result<types::FactMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO facts_memories (
                fact_id,
                memory_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING
                fact_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.fact_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::FactMemoryRow) -> Result<types::FactMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE facts_memories
            SET
                created_at = $3
            WHERE fact_id = $1 AND memory_id = $2
            RETURNING
                fact_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.fact_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM facts_memories
            WHERE fact_id = $1 AND memory_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
