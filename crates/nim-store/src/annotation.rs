use crate::types;

pub struct AnnotationStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> AnnotationStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::AnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                memory_id,
                label,
                text,
                spans,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM annotations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::AnnotationRow) -> Result<types::AnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO annotations (
                id,
                memory_id,
                label,
                text,
                spans,
                confidence,
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
                $6,
                $7,
                $8,
                $9
            )
            RETURNING
                id,
                memory_id,
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
        .bind(value.memory_id)
        .bind(value.label)
        .bind(value.text)
        .bind(value.spans)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::AnnotationRow) -> Result<types::AnnotationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE annotations
            SET
                memory_id = $2,
                label = $3,
                text = $4,
                spans = $5,
                confidence = $6,
                embedding = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING
                id,
                memory_id,
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
        .bind(value.memory_id)
        .bind(value.label)
        .bind(value.text)
        .bind(value.spans)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM annotations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
