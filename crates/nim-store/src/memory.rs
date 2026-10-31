use crate::{MemoryFilter, MemoryType, SearchItem, types};

#[derive(Copy, Clone)]
pub struct MemoryStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MemoryStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<types::MemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                id,
                version,
                type,
                sources,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
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

    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: uuid::Uuid) -> Result<Vec<types::MemoryRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                memories.id,
                memories.version,
                memories.type,
                memories.sources,
                memories.contexts,
                memories.salience,
                memories.strength,
                memories.confidence,
                memories.recalls,
                memories.recalled_at,
                memories.created_at,
                memories.updated_at
            FROM memories
            JOIN memories_artifacts ON memories_artifacts.memory_id = memories.id
            WHERE memories_artifacts.artifact_id = $1
            ORDER BY memories.id
            "#,
        )
        .bind(artifact_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_by_relation_id(&self, relation_id: uuid::Uuid) -> Result<Vec<types::MemoryRow>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                memories.id,
                memories.version,
                memories.type,
                memories.sources,
                memories.contexts,
                memories.salience,
                memories.strength,
                memories.confidence,
                memories.recalls,
                memories.recalled_at,
                memories.created_at,
                memories.updated_at
            FROM memories
            JOIN memories_relations ON memories_relations.source_id = memories.id
            WHERE memories_relations.target_id = $1
            ORDER BY memories.id
            "#,
        )
        .bind(relation_id)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip_all)]
    pub async fn search(
        &self,
        embedding: &[f32],
        filter: MemoryFilter,
    ) -> Result<Vec<SearchItem<types::MemoryRow>>, sqlx::Error> {
        #[derive(askama::Template)]
        #[template(
            ext = "txt",
            source = "
                SELECT
                    (1 - LEAST(artifacts.distance, annotations.distance))::real AS similarity,
                    memories.*
                FROM memories
                LEFT JOIN (
                    SELECT memories_artifacts.memory_id, MIN(artifacts.embedding <=> $1::vector) AS distance
                    FROM artifacts
                    JOIN memories_artifacts ON memories_artifacts.artifact_id = artifacts.id
                    WHERE artifacts.embedding IS NOT NULL
                    GROUP BY memories_artifacts.memory_id
                ) AS artifacts ON artifacts.memory_id = memories.id
                LEFT JOIN (
                    SELECT memories_artifacts.memory_id, MIN(annotations.embedding <=> $1::vector) AS distance
                    FROM annotations
                    JOIN memories_artifacts ON memories_artifacts.artifact_id = annotations.artifact_id
                    WHERE annotations.embedding IS NOT NULL
                    GROUP BY memories_artifacts.memory_id
                ) AS annotations ON annotations.memory_id = memories.id
                WHERE (artifacts.distance IS NOT NULL OR annotations.distance IS NOT NULL)
                {% if ty.is_some() %}
                AND memories.type = $2
                {% endif %}
                AND (1 - LEAST(artifacts.distance, annotations.distance))::real >= $4
                ORDER BY LEAST(artifacts.distance, annotations.distance), memories.id
                LIMIT $3;
            "
        )]
        struct Query {
            ty: Option<MemoryType>,
        }

        let query = Query { ty: filter.ty };

        sqlx::query_as(sqlx::AssertSqlSafe(query.to_string()))
            .bind(pgvector::Vector::from(embedding.to_vec()))
            .bind(filter.ty)
            .bind(filter.limit.map(|v| v as i64).unwrap_or(10))
            .bind(filter.min_score.unwrap_or(0.2))
            .fetch_all(self.pool)
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MemoryRow) -> Result<types::MemoryRow, sqlx::Error> {
        sqlx::query_as(
            r#"
            INSERT INTO memories (
                id,
                version,
                type,
                sources,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
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
                $12
            )
            RETURNING
                id,
                version,
                type,
                sources,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
                recalled_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.version)
        .bind(value.ty)
        .bind(value.sources)
        .bind(value.contexts)
        .bind(value.salience)
        .bind(value.strength)
        .bind(value.confidence)
        .bind(value.recalls)
        .bind(value.recalled_at)
        .bind(value.created_at)
        .bind(value.updated_at)
        .fetch_one(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update(&self, value: types::MemoryRow) -> Result<types::MemoryRow, sqlx::Error> {
        let row = sqlx::query_as(
            r#"
            UPDATE memories
            SET
                version = version + 1,
                type = $2,
                sources = $3,
                contexts = $4,
                salience = GREATEST(salience, $5),
                strength = $6,
                confidence = $7,
                recalls = $8,
                recalled_at = $9,
                updated_at = $10
            WHERE id = $1 AND version = $11
            RETURNING
                id,
                version,
                type,
                sources,
                contexts,
                salience,
                strength,
                confidence,
                recalls,
                recalled_at,
                created_at,
                updated_at
            "#,
        )
        .bind(value.id)
        .bind(value.ty)
        .bind(value.sources)
        .bind(value.contexts)
        .bind(value.salience)
        .bind(value.strength)
        .bind(value.confidence)
        .bind(value.recalls)
        .bind(value.recalled_at)
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

#[derive(Copy, Clone)]
pub struct MemoryRelationsStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> MemoryRelationsStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: types::MemoryRelationRow) -> Result<types::MemoryRelationRow, sqlx::Error> {
        let row = sqlx::query_as(
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
            ON CONFLICT DO NOTHING
            RETURNING
                source_id,
                target_id,
                created_at
            "#,
        )
        .bind(value.source_id)
        .bind(value.target_id)
        .bind(value.created_at)
        .fetch_optional(self.pool)
        .await?;

        match row {
            Some(row) => Ok(row),
            None => self.find_by_id(value.source_id, value.target_id).await,
        }
    }

    #[tracing::instrument(skip(self))]
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
