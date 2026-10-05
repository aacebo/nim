use crate::types;

pub struct GoalStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> GoalStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::GoalRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                parent_id,
                version,
                description,
                status,
                priority,
                deadline,
                conditions,
                created_at,
                updated_at
            FROM goals
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::GoalRow) -> Result<types::GoalRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO goals (
                id,
                parent_id,
                version,
                description,
                status,
                priority,
                deadline,
                conditions,
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
                parent_id,
                version,
                description,
                status,
                priority,
                deadline,
                conditions,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.parent_id)
        .bind(value.version)
        .bind(value.description)
        .bind(value.status)
        .bind(value.priority)
        .bind(value.deadline)
        .bind(value.conditions)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::GoalRow) -> Result<types::GoalRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE goals
            SET
                parent_id = $2,
                version = version + 1,
                description = $3,
                status = $4,
                priority = $5,
                deadline = $6,
                conditions = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING
                id,
                parent_id,
                version,
                description,
                status,
                priority,
                deadline,
                conditions,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.parent_id)
        .bind(value.description)
        .bind(value.status)
        .bind(value.priority)
        .bind(value.deadline)
        .bind(value.conditions)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM goals
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
