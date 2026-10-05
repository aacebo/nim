use crate::types;

pub struct PlanStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> PlanStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::PlanRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                title,
                status,
                description,
                created_at,
                updated_at
            FROM plans
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::PlanRow) -> Result<types::PlanRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO plans (
                id,
                title,
                status,
                description,
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
                title,
                status,
                description,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.title)
        .bind(value.status)
        .bind(value.description)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::PlanRow) -> Result<types::PlanRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE plans
            SET
                title = $2,
                status = $3,
                description = $4,
                updated_at = $5
            WHERE id = $1
            RETURNING
                id,
                title,
                status,
                description,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.title)
        .bind(value.status)
        .bind(value.description)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM plans
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

pub struct StepStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> StepStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::StepRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                plan_id,
                position,
                name,
                status,
                about,
                action,
                condition,
                created_at,
                updated_at
            FROM steps
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::StepRow) -> Result<types::StepRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO steps (
                id,
                plan_id,
                position,
                name,
                status,
                about,
                action,
                condition,
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
                plan_id,
                position,
                name,
                status,
                about,
                action,
                condition,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.plan_id)
        .bind(value.position)
        .bind(value.name)
        .bind(value.status)
        .bind(value.about)
        .bind(value.action)
        .bind(value.condition)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::StepRow) -> Result<types::StepRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE steps
            SET
                plan_id = $2,
                position = $3,
                name = $4,
                status = $5,
                about = $6,
                action = $7,
                condition = $8,
                updated_at = $9
            WHERE id = $1
            RETURNING
                id,
                plan_id,
                position,
                name,
                status,
                about,
                action,
                condition,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.plan_id)
        .bind(value.position)
        .bind(value.name)
        .bind(value.status)
        .bind(value.about)
        .bind(value.action)
        .bind(value.condition)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM steps
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
