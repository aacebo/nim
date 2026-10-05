mod action;
mod condition;
mod status;

pub use action::*;
pub use condition::*;
pub use status::*;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlanRow {
    pub id: uuid::Uuid,
    pub title: String,
    pub status: PlanStatus,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StepRow {
    pub id: uuid::Uuid,
    pub plan_id: uuid::Uuid,
    pub position: i32,
    pub name: String,
    pub status: StepStatus,
    pub about: Option<String>,
    pub action: sqlx::types::Json<Action>,
    pub condition: Option<sqlx::types::Json<Condition>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
