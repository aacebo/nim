#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GoalRow {
    pub id: uuid::Uuid,
    pub parent_id: uuid::Uuid,
    pub version: i32,
    pub description: String,
    pub status: GoalStatus,
    pub priority: f64,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub conditions: sqlx::types::Json<Vec<GoalCondition>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum GoalStatus {
    Active,
    Suspended,
    Completed,
    Failed,
    Cancelled,
}

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GoalCondition {
    EntityState { entity: uuid::Uuid, state: String },
    Fact { description: String },
    Time { before: chrono::DateTime<chrono::Utc> },
    Custom { description: String },
}
