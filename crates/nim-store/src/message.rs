use crate::{Page, types};

#[derive(Copy, Clone)]
pub struct MessageStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MessageStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::MessageRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                chat_id,
                parent_id,
                version,
                role,
                content,
                summary,
                embedding,
                confidence,
                created_at,
                updated_at
            FROM messages
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: uuid::Uuid, page: Page) -> Result<Vec<types::MessageRow>, sqlx::Error> {
        let (cursor_id, cursor_at) = page.cursor.map(|cursor| (cursor.id, cursor.timestamp)).unzip();

        sqlx::query_as(
            r#"
            SELECT
                id,
                chat_id,
                parent_id,
                version,
                role,
                content,
                summary,
                embedding,
                confidence,
                created_at,
                updated_at
            FROM messages
            WHERE chat_id = $1
            AND ($2::timestamptz IS NULL OR (created_at, id) > ($2, $3::uuid))
            ORDER BY created_at, id
            LIMIT $4
            "#,
        )
        .bind(chat_id)
        .bind(cursor_at)
        .bind(cursor_id)
        .bind(i64::try_from(page.limit).unwrap_or(i64::MAX))
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_parent_id(&self, parent_id: uuid::Uuid, page: Page) -> Result<Vec<types::MessageRow>, sqlx::Error> {
        let (cursor_id, cursor_at) = page.cursor.map(|cursor| (cursor.id, cursor.timestamp)).unzip();

        sqlx::query_as(
            r#"
            SELECT
                id,
                chat_id,
                parent_id,
                version,
                role,
                content,
                summary,
                embedding,
                confidence,
                created_at,
                updated_at
            FROM messages
            WHERE parent_id = $1
            AND ($2::timestamptz IS NULL OR (created_at, id) > ($2, $3::uuid))
            ORDER BY created_at, id
            LIMIT $4
            "#,
        )
        .bind(parent_id)
        .bind(cursor_at)
        .bind(cursor_id)
        .bind(i64::try_from(page.limit).unwrap_or(i64::MAX))
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MessageRow) -> Result<types::MessageRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO messages (
                id,
                chat_id,
                parent_id,
                version,
                role,
                content,
                summary,
                embedding,
                confidence,
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
                $10,
                $11
            )
            RETURNING
                id,
                chat_id,
                parent_id,
                version,
                role,
                content,
                summary,
                embedding,
                confidence,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.chat_id)
        .bind(value.parent_id)
        .bind(value.version)
        .bind(value.role)
        .bind(value.content)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.confidence)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::MessageRow) -> Result<types::MessageRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE messages
            SET
                version = version + 1,
                content = $2,
                summary = $3,
                embedding = $4,
                confidence = $5,
                updated_at = $6
            WHERE id = $1 AND version = $7
            RETURNING
                id,
                chat_id,
                parent_id,
                version,
                role,
                content,
                summary,
                embedding,
                confidence,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.content)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.confidence)
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
            DELETE FROM messages
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
pub struct MessageAnnotationStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MessageAnnotationStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, message_id: uuid::Uuid, id: uuid::Uuid) -> Result<types::MessageAnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                message_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM message_annotations
            WHERE id = $1 AND message_id = $2
            "#,
        )
        .bind(id)
        .bind(message_id)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_message_id(&self, message_id: uuid::Uuid) -> Result<Vec<types::MessageAnnotationRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                message_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM message_annotations
            WHERE message_id = $1
            "#,
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_message_label(
        &self,
        message_id: uuid::Uuid,
        label: types::Label,
        text: &str,
    ) -> Result<Option<types::MessageAnnotationRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                message_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM message_annotations
            WHERE message_id = $1
            AND label = $2
            AND text = $3
            "#,
        )
        .bind(message_id)
        .bind(label)
        .bind(text)
        .fetch_optional(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MessageAnnotationRow) -> Result<types::MessageAnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO message_annotations (
                id,
                message_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at,
                version
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
                message_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.message_id)
        .bind(value.label)
        .bind(value.text)
        .bind(value.spans)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.created_at)
        .bind(value.updated_at)
        .bind(value.version)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::MessageAnnotationRow) -> Result<types::MessageAnnotationRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE message_annotations
            SET
                version = version + 1,
                label = $2,
                text = $3,
                spans = $4,
                confidence = $5,
                embedding = $6,
                updated_at = $7
            WHERE id = $1 AND version = $8 AND message_id = $9
            RETURNING
                id,
                version,
                message_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.label)
        .bind(value.text)
        .bind(value.spans)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(chrono::Utc::now())
        .bind(value.version)
        .bind(value.message_id)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.message_id, value.id).await,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, message_id: uuid::Uuid, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM message_annotations
            WHERE id = $1 AND message_id = $2
            "#,
        )
        .bind(id)
        .bind(message_id)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }
}
