use crate::types;

pub struct EntityStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> EntityStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::EntityRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                type,
                version,
                name,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            FROM entities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::EntityRow) -> Result<types::EntityRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO entities (
                id,
                type,
                version,
                name,
                summary,
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
                type,
                version,
                name,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.ty)
        .bind(value.version)
        .bind(value.name)
        .bind(value.summary)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::EntityRow) -> Result<types::EntityRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE entities
            SET
                type = $2,
                version = version + 1,
                name = $3,
                summary = $4,
                confidence = $5,
                embedding = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING
                id,
                type,
                version,
                name,
                summary,
                confidence,
                embedding,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.ty)
        .bind(value.name)
        .bind(value.summary)
        .bind(value.confidence)
        .bind(value.embedding)
        .bind(chrono::Utc::now())
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM entities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

pub struct EntityMemoriesStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> EntityMemoriesStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, entity_id: uuid::Uuid, memory_id: uuid::Uuid) -> Result<types::EntityMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                entity_id,
                memory_id,
                created_at
            FROM entities_memories
            WHERE entity_id = $1 AND memory_id = $2
            "#,
        )
        .bind(entity_id)
        .bind(memory_id)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::EntityMemoryRow) -> Result<types::EntityMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO entities_memories (
                entity_id,
                memory_id,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3
            )
            RETURNING
                entity_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.entity_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::EntityMemoryRow) -> Result<types::EntityMemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE entities_memories
            SET
                created_at = $3
            WHERE entity_id = $1 AND memory_id = $2
            RETURNING
                entity_id,
                memory_id,
                created_at
            "#,
        )
        .bind(value.entity_id)
        .bind(value.memory_id)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM entities_memories
            WHERE entity_id = $1 AND memory_id = $2
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

pub struct EntityRelationsStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> EntityRelationsStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        source_id: uuid::Uuid,
        target_id: uuid::Uuid,
        ty: types::EntityRelationType,
    ) -> Result<types::EntityRelationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                source_id,
                target_id,
                type,
                created_at
            FROM entities_relations
            WHERE source_id = $1 AND target_id = $2 AND type = $3
            "#,
        )
        .bind(source_id)
        .bind(target_id)
        .bind(ty)
        .fetch_one(self.pool)
        .await
    }

    pub async fn create(&self, value: types::EntityRelationRow) -> Result<types::EntityRelationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO entities_relations (
                source_id,
                target_id,
                type,
                created_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING
                source_id,
                target_id,
                type,
                created_at
            "#,
        )
        .bind(value.source_id)
        .bind(value.target_id)
        .bind(value.ty)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, value: types::EntityRelationRow) -> Result<types::EntityRelationRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            UPDATE entities_relations
            SET
                created_at = $4
            WHERE source_id = $1 AND target_id = $2 AND type = $3
            RETURNING
                source_id,
                target_id,
                type,
                created_at
            "#,
        )
        .bind(value.source_id)
        .bind(value.target_id)
        .bind(value.ty)
        .bind(value.created_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: (uuid::Uuid, uuid::Uuid, types::EntityRelationType)) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM entities_relations
            WHERE source_id = $1 AND target_id = $2 AND type = $3
            "#,
        )
        .bind(id.0)
        .bind(id.1)
        .bind(id.2)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
