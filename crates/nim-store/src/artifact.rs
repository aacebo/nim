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
                memory_id,
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
                id,
                memory_id,
                name,
                content,
                created_at,
                updated_at
            FROM artifacts
            WHERE memory_id = $1
            ORDER BY created_at, id
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
                memory_id,
                type,
                name,
                content,
                created_at,
                updated_at
            )
            VALUES (
                $1,
                $2,
                $4::jsonb ->> 'type',
                $3,
                $4,
                $5,
                $6
            )
            RETURNING
                id,
                memory_id,
                name,
                content,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.memory_id)
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
                memory_id = $2,
                type = $4::jsonb ->> 'type',
                name = $3,
                content = $4,
                updated_at = $5
            WHERE id = $1
            RETURNING
                id,
                memory_id,
                name,
                content,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.memory_id)
        .bind(value.name)
        .bind(value.content)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
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
