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
                type,
                name,
                tasks,
                architectures,
                source,
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
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn upsert(&self, value: types::ModelRow) -> Result<types::ModelRow, sqlx::Error> {
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
            ON CONFLICT (name) DO UPDATE
            SET
                version = models.version + 1,
                type = EXCLUDED.type,
                architectures = EXCLUDED.architectures,
                source = EXCLUDED.source,
                updated_at = EXCLUDED.updated_at
            RETURNING
                id,
                version,
                type,
                name,
                tasks,
                architectures,
                source,
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
                updated_at = $7
            WHERE id = $1 AND version = $8
            RETURNING
                id,
                version,
                type,
                name,
                tasks,
                architectures,
                source,
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

#[derive(Copy, Clone)]
pub struct ModelWeightStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ModelWeightStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, model_id: uuid::Uuid, id: uuid::Uuid) -> Result<types::ModelWeightRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                model_id,
                version,
                name,
                content,
                created_at,
                updated_at
            FROM model_weights
            WHERE model_id = $1 AND id = $2
            "#,
        )
        .bind(model_id)
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_model_id(&self, model_id: uuid::Uuid) -> Result<Vec<types::ModelWeightRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                model_id,
                version,
                name,
                content,
                created_at,
                updated_at
            FROM model_weights
            WHERE model_id = $1
            ORDER BY created_at, id
            "#,
        )
        .bind(model_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ModelWeightRow) -> Result<types::ModelWeightRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO model_weights (
                id,
                model_id,
                version,
                name,
                content,
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
                model_id,
                version,
                name,
                content,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.model_id)
        .bind(value.version)
        .bind(value.name)
        .bind(value.content)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::ModelWeightRow) -> Result<types::ModelWeightRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE model_weights
            SET
                version = version + 1,
                name = $3,
                content = $4,
                updated_at = $5
            WHERE model_id = $1 AND id = $2 AND version = $6
            RETURNING
                id,
                model_id,
                version,
                name,
                content,
                created_at,
                updated_at
            "#,
        )
        .bind(value.model_id)
        .bind(value.id)
        .bind(value.name)
        .bind(value.content)
        .bind(chrono::Utc::now())
        .bind(value.version)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.model_id, value.id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, model_id: uuid::Uuid, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM model_weights
            WHERE model_id = $1 AND id = $2
            "#,
        )
        .bind(model_id)
        .bind(id)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}
