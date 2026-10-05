#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    GoalCompleted { goal: uuid::Uuid },
    EntityState { entity: uuid::Uuid, state: String },
    Observation { description: String },
    Time { after: chrono::DateTime<chrono::Utc> },
}
