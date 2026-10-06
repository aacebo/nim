#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EntityRow {
    pub id: uuid::Uuid,
    #[sqlx(rename = "type")]
    pub ty: EntityType,
    pub version: i32,
    pub name: String,
    pub summary: Option<String>,
    pub confidence: f64,
    pub embedding: Option<pgvector::Vector>,
    pub recalls: i64,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EntityMemoryRow {
    pub entity_id: uuid::Uuid,
    pub memory_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EntityRelationRow {
    pub source_id: uuid::Uuid,
    pub target_id: uuid::Uuid,
    #[sqlx(rename = "type")]
    pub ty: EntityRelationType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Place,
    Organization,
    Project,
    Object,
    Event,
    Other,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum EntityRelationType {
    PartOf,
    MemberOf,
    LocatedIn,
    WorksOn,
    Owns,
    RelatedTo,
}
