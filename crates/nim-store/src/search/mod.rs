mod filters;
mod item;

pub use filters::*;
pub use item::*;

use crate::types;

#[derive(Copy, Clone)]
pub struct SearchStorage<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> SearchStorage<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(skip_all)]
    pub async fn memories(
        &self,
        embedding: &[f32],
        filter: MemoryFilter,
    ) -> Result<Vec<SearchItem<types::MemoryRow>>, sqlx::Error> {
        sqlx::query_as(
            r#"
                SELECT
                    (1 - LEAST(artifacts.distance, annotations.distance))::real AS similarity,
                    to_jsonb(memories) AS "inner"
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
                    FROM artifact_annotations AS annotations
                    JOIN memories_artifacts ON memories_artifacts.artifact_id = annotations.artifact_id
                    WHERE annotations.embedding IS NOT NULL
                    GROUP BY memories_artifacts.memory_id
                ) AS annotations ON annotations.memory_id = memories.id
                WHERE (artifacts.distance IS NOT NULL OR annotations.distance IS NOT NULL)
                AND ($2::text IS NULL OR memories.type = $2)
                AND (1 - LEAST(artifacts.distance, annotations.distance))::real >= $4
                ORDER BY similarity DESC, memories.created_at DESC, memories.id
                LIMIT $3;
            "#,
        )
        .bind(pgvector::Vector::from(embedding.to_vec()))
        .bind(filter.ty)
        .bind(filter.limit.map(|v| v as i64).unwrap_or(10))
        .bind(filter.min_score.unwrap_or(0.2))
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn chats(&self, embedding: &[f32], filter: ChatFilter) -> Result<Vec<SearchItem<types::ChatRow>>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                (1 - (embedding <=> $1::vector))::real AS similarity,
                (to_jsonb(chats) || jsonb_build_object('embedding', chats.embedding::text::jsonb)) AS "inner"
            FROM chats
            WHERE embedding IS NOT NULL
            AND (1 - (embedding <=> $1::vector))::real >= $2
            ORDER BY similarity DESC, created_at DESC, id
            LIMIT $3
            "#,
        )
        .bind(pgvector::Vector::from(embedding.to_vec()))
        .bind(filter.min_score.unwrap_or(0.2))
        .bind(filter.limit.map(|value| value as i64).unwrap_or(10))
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn messages(
        &self,
        embedding: &[f32],
        filter: MessageFilter,
    ) -> Result<Vec<SearchItem<types::MessageRow>>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                (1 - (embedding <=> $1::vector))::real AS similarity,
                (to_jsonb(messages) || jsonb_build_object('embedding', messages.embedding::text::jsonb)) AS "inner"
            FROM messages
            WHERE embedding IS NOT NULL
            AND (1 - (embedding <=> $1::vector))::real >= $2
            AND ($4::uuid IS NULL OR chat_id = $4)
            AND ($5::text IS NULL OR role = $5)
            ORDER BY similarity DESC, created_at DESC, id
            LIMIT $3
            "#,
        )
        .bind(pgvector::Vector::from(embedding.to_vec()))
        .bind(filter.min_score.unwrap_or(0.2))
        .bind(filter.limit.map(|value| value as i64).unwrap_or(10))
        .bind(filter.chat_id)
        .bind(filter.role)
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn artifacts(&self, embedding: &[f32], filter: Filter) -> Result<Vec<SearchItem<types::ArtifactRow>>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
                (1 - LEAST(artifacts.embedding <=> $1::vector, annotations.distance))::real AS similarity,
                (to_jsonb(artifacts) || jsonb_build_object('embedding', artifacts.embedding::text::jsonb)) AS "inner"
            FROM artifacts
            LEFT JOIN (
                SELECT artifact_id, MIN(embedding <=> $1::vector) AS distance
                FROM artifact_annotations
                WHERE embedding IS NOT NULL
                GROUP BY artifact_id
            ) AS annotations ON annotations.artifact_id = artifacts.id
            WHERE (artifacts.embedding IS NOT NULL OR annotations.distance IS NOT NULL)
            AND (1 - LEAST(artifacts.embedding <=> $1::vector, annotations.distance))::real >= $2
            ORDER BY similarity DESC, artifacts.created_at DESC, artifacts.id
            LIMIT $3
            "#,
        )
        .bind(pgvector::Vector::from(embedding.to_vec()))
        .bind(filter.min_score.unwrap_or(0.2))
        .bind(filter.limit.map(|value| value as i64).unwrap_or(10))
        .fetch_all(self.pool)
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn all(&self, embedding: &[f32], filter: Filter) -> Result<Vec<SearchItem<types::RecordRow>>, sqlx::Error> {
        sqlx::query_as(
            r#"
            WITH artifact_distances AS (
                SELECT artifacts.*, LEAST(artifacts.embedding <=> $1::vector, annotations.distance) AS distance
                FROM artifacts
                LEFT JOIN (
                    SELECT artifact_id, MIN(embedding <=> $1::vector) AS distance
                    FROM artifact_annotations
                    WHERE embedding IS NOT NULL
                    GROUP BY artifact_id
                ) AS annotations ON annotations.artifact_id = artifacts.id
            ), records AS (
                SELECT 'artifact'::text AS record_type, artifacts.id, artifacts.created_at,
                    (1 - artifacts.distance)::real AS similarity,
                    (to_jsonb(artifacts) || jsonb_build_object('embedding', artifacts.embedding::text::jsonb)) - 'distance' AS "inner"
                FROM artifact_distances AS artifacts
                UNION ALL
                SELECT 'chat'::text AS record_type, chats.id, chats.created_at,
                    (1 - (chats.embedding <=> $1::vector))::real AS similarity,
                    (to_jsonb(chats) || jsonb_build_object('embedding', chats.embedding::text::jsonb)) AS "inner"
                FROM chats
                WHERE chats.embedding IS NOT NULL
                UNION ALL
                SELECT 'message'::text AS record_type, messages.id, messages.created_at,
                    (1 - (messages.embedding <=> $1::vector))::real AS similarity,
                    (to_jsonb(messages) || jsonb_build_object('embedding', messages.embedding::text::jsonb)) AS "inner"
                FROM messages
                WHERE messages.embedding IS NOT NULL
                UNION ALL
                SELECT 'memory'::text AS record_type, memories.id, memories.created_at,
                    (1 - MIN(artifacts.distance))::real AS similarity,
                    to_jsonb(memories) AS "inner"
                FROM memories
                JOIN memories_artifacts ON memories_artifacts.memory_id = memories.id
                JOIN artifact_distances AS artifacts ON artifacts.id = memories_artifacts.artifact_id
                GROUP BY memories.id
            )
            SELECT similarity, "inner" FROM records
            WHERE similarity >= $2
            ORDER BY similarity DESC, created_at DESC, record_type, id
            LIMIT $3
            "#,
        )
        .bind(pgvector::Vector::from(embedding.to_vec()))
        .bind(filter.min_score.unwrap_or(0.2))
        .bind(filter.limit.map(|value| value as i64).unwrap_or(10))
        .fetch_all(self.pool)
        .await
    }
}
