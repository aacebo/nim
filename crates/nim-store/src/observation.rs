use crate::types;

pub struct ObservationStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ObservationStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::ObservationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                source,
                description,
                embedding,
                created_at,
                updated_at
            FROM observations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::ObservationRow) -> Result<types::ObservationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO observations (
                id,
                source,
                description,
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
                $6
            )
            RETURNING
                id,
                source,
                description,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.source)
        .bind(value.description)
        .bind(value.embedding)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::ObservationRow) -> Result<types::ObservationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE observations
            SET
                source = $2,
                description = $3,
                embedding = $4,
                updated_at = $5
            WHERE id = $1
            RETURNING
                id,
                source,
                description,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.source)
        .bind(value.description)
        .bind(value.embedding)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM observations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

pub struct ObservationMemoriesStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ObservationMemoriesStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        observation_id: uuid::Uuid,
        memory_id: uuid::Uuid,
    ) -> Result<types::ObservationMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                observation_id,
                memory_id,
                created_at
            FROM observations_memories
            WHERE observation_id = $1 AND memory_id = $2
            "#,
        )
        .bind(observation_id)
        .bind(memory_id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::ObservationMemoryRow) -> Result<types::ObservationMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO observations_memories (
                observation_id,
                memory_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING
                observation_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.observation_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::ObservationMemoryRow) -> Result<types::ObservationMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE observations_memories
            SET
                created_at = $3
            WHERE observation_id = $1 AND memory_id = $2
            RETURNING
                observation_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.observation_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM observations_memories
            WHERE observation_id = $1 AND memory_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
