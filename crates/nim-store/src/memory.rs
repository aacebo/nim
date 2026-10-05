use crate::types;

pub struct MemoryStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MemoryStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::MemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                type,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
                description,
                summary,
                embedding,
                recalled_at,
                created_at,
                updated_at
            FROM memories
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::MemoryRow) -> Result<types::MemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO memories (
                id,
                version,
                type,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
                description,
                summary,
                embedding,
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
                $8,
                $9,
                $10,
                $11,
                $12,
                $13,
                $14
            )
            RETURNING
                id,
                version,
                type,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
                description,
                summary,
                embedding,
                recalled_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.version)
        .bind(value.ty)
        .bind(value.contexts)
        .bind(value.salience)
        .bind(value.strength)
        .bind(value.confidence)
        .bind(value.recalls)
        .bind(value.description)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.recalled_at)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::MemoryRow) -> Result<types::MemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE memories
            SET
                version = version + 1,
                type = $2,
                contexts = $3,
                salience = $4,
                strength = $5,
                confidence = $6,
                recalls = $7,
                description = $8,
                summary = $9,
                embedding = $10,
                recalled_at = $11,
                updated_at = $12
            WHERE id = $1
            RETURNING
                id,
                version,
                type,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
                description,
                summary,
                embedding,
                recalled_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.ty)
        .bind(value.contexts)
        .bind(value.salience)
        .bind(value.strength)
        .bind(value.confidence)
        .bind(value.recalls)
        .bind(value.description)
        .bind(value.summary)
        .bind(value.embedding)
        .bind(value.recalled_at)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM memories
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(())
    }
}

pub struct MemoryRelationsStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MemoryRelationsStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        source_id: uuid::Uuid,
        target_id: uuid::Uuid,
    ) -> Result<types::MemoryRelationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                source_id,
                target_id,
                created_at
            FROM memories_relations
            WHERE source_id = $1 AND target_id = $2
            "#,
        )
        .bind(source_id)
        .bind(target_id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::MemoryRelationRow) -> Result<types::MemoryRelationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO memories_relations (
                source_id,
                target_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING
                source_id,
                target_id,
                created_at
            "#,
        )
        .bind(value.source_id)
        .bind(value.target_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::MemoryRelationRow) -> Result<types::MemoryRelationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE memories_relations
            SET
                created_at = $3
            WHERE source_id = $1 AND target_id = $2
            RETURNING
                source_id,
                target_id,
                created_at
            "#,
        )
        .bind(value.source_id)
        .bind(value.target_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM memories_relations
            WHERE source_id = $1 AND target_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
