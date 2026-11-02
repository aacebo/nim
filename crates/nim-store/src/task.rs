use crate::types;

#[derive(Copy, Clone)]
pub struct TaskStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> TaskStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::TaskRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                name,
                source,
                status,
                created_at,
                updated_at,
                parent_id
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::TaskRow) -> Result<types::TaskRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO tasks (
                id,
                version,
                name,
                source,
                status,
                created_at,
                updated_at,
                parent_id
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
                version,
                name,
                source,
                status,
                created_at,
                updated_at,
                parent_id
            "#,
        )
        .bind(value.id)
        .bind(value.version)
        .bind(value.name)
        .bind(value.source)
        .bind(value.status)
        .bind(value.created_at)
        .bind(value.updated_at)
        .bind(value.parent_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::TaskRow) -> Result<types::TaskRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE tasks
            SET
                version = version + 1,
                name = $2,
                source = $3,
                status = $4,
                updated_at = $5,
                parent_id = $7
            WHERE id = $1 AND version = $6
            RETURNING
                id,
                version,
                name,
                source,
                status,
                created_at,
                updated_at,
                parent_id
            "#,
        )
        .bind(value.id)
        .bind(value.name)
        .bind(value.source)
        .bind(value.status)
        .bind(chrono::Utc::now())
        .bind(value.version)
        .bind(value.parent_id)
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
            DELETE FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_model_id(&self, model_id: uuid::Uuid) -> Result<Vec<types::TaskRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT tasks.*
            FROM tasks
            JOIN models_tasks ON models_tasks.task_id = tasks.id
            WHERE models_tasks.model_id = $1
            "#,
        )
        .bind(model_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: uuid::Uuid) -> Result<Vec<types::TaskRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT tasks.*
            FROM tasks
            JOIN artifacts_tasks ON artifacts_tasks.task_id = tasks.id
            WHERE artifacts_tasks.artifact_id = $1
            "#,
        )
        .bind(artifact_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: uuid::Uuid) -> Result<Vec<types::TaskRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT tasks.*
            FROM tasks
            JOIN chats_tasks ON chats_tasks.task_id = tasks.id
            WHERE chats_tasks.chat_id = $1
            "#,
        )
        .bind(chat_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_message_id(&self, message_id: uuid::Uuid) -> Result<Vec<types::TaskRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT tasks.*
            FROM tasks
            JOIN messages_tasks ON messages_tasks.task_id = tasks.id
            WHERE messages_tasks.message_id = $1
            "#,
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }
}

#[derive(Copy, Clone)]
pub struct ModelTasksStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ModelTasksStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, model_id: uuid::Uuid, task_id: uuid::Uuid) -> Result<types::ModelTaskRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                model_id,
                task_id,
                created_at
            FROM models_tasks
            WHERE model_id = $1 AND task_id = $2
            "#,
        )
        .bind(model_id)
        .bind(task_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ModelTaskRow) -> Result<types::ModelTaskRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO models_tasks (
                model_id,
                task_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            ON CONFLICT DO NOTHING
            RETURNING
                model_id,
                task_id,
                created_at
            "#,
        )
        .bind(value.model_id)
        .bind(value.task_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.model_id, value.task_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM models_tasks
            WHERE model_id = $1 AND task_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct ArtifactTasksStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ArtifactTasksStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, artifact_id: uuid::Uuid, task_id: uuid::Uuid) -> Result<types::ArtifactTaskRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                artifact_id,
                task_id,
                created_at
            FROM artifacts_tasks
            WHERE artifact_id = $1 AND task_id = $2
            "#,
        )
        .bind(artifact_id)
        .bind(task_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ArtifactTaskRow) -> Result<types::ArtifactTaskRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO artifacts_tasks (
                artifact_id,
                task_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            ON CONFLICT DO NOTHING
            RETURNING
                artifact_id,
                task_id,
                created_at
            "#,
        )
        .bind(value.artifact_id)
        .bind(value.task_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.artifact_id, value.task_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM artifacts_tasks
            WHERE artifact_id = $1 AND task_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct ChatTasksStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> ChatTasksStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, chat_id: uuid::Uuid, task_id: uuid::Uuid) -> Result<types::ChatTaskRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                chat_id,
                task_id,
                created_at
            FROM chats_tasks
            WHERE chat_id = $1 AND task_id = $2
            "#,
        )
        .bind(chat_id)
        .bind(task_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::ChatTaskRow) -> Result<types::ChatTaskRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO chats_tasks (
                chat_id,
                task_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            ON CONFLICT DO NOTHING
            RETURNING
                chat_id,
                task_id,
                created_at
            "#,
        )
        .bind(value.chat_id)
        .bind(value.task_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.chat_id, value.task_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM chats_tasks
            WHERE chat_id = $1 AND task_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct MessageTasksStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MessageTasksStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, message_id: uuid::Uuid, task_id: uuid::Uuid) -> Result<types::MessageTaskRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                message_id,
                task_id,
                created_at
            FROM messages_tasks
            WHERE message_id = $1 AND task_id = $2
            "#,
        )
        .bind(message_id)
        .bind(task_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MessageTaskRow) -> Result<types::MessageTaskRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO messages_tasks (
                message_id,
                task_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            ON CONFLICT DO NOTHING
            RETURNING
                message_id,
                task_id,
                created_at
            "#,
        )
        .bind(value.message_id)
        .bind(value.task_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.message_id, value.task_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM messages_tasks
            WHERE message_id = $1 AND task_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
