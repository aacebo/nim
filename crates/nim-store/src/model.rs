use crate::types;

#[derive(Copy, Clone)]
pub struct ModelStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ModelStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::ModelRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                type,
                name,
                tasks,
                architectures,
                source,
                weights,
                created_at,
                updated_at
            FROM models
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_name(&self, name: &str) -> Result<types::ModelRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                type,
                name,
                tasks,
                architectures,
                source,
                weights,
                created_at,
                updated_at
            FROM models
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ModelRow) -> Result<types::ModelRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO models (
                id,
                version,
                type,
                name,
                tasks,
                architectures,
                source,
                weights,
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
                version,
                type,
                name,
                tasks,
                architectures,
                source,
                weights,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.version)
        .bind(value.ty)
        .bind(value.name)
        .bind(value.tasks)
        .bind(value.architectures)
        .bind(value.source)
        .bind(value.weights)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::ModelRow) -> Result<types::ModelRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE models
            SET
                version = version + 1,
                type = $2,
                name = $3,
                tasks = $4,
                architectures = $5,
                source = $6,
                weights = $7,
                updated_at = $8
            WHERE id = $1 AND version = $9
            RETURNING
                id,
                version,
                type,
                name,
                tasks,
                architectures,
                source,
                weights,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.ty)
        .bind(value.name)
        .bind(value.tasks)
        .bind(value.architectures)
        .bind(value.source)
        .bind(value.weights)
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
            DELETE FROM models
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
