use crate::{EntityId, GoalId};

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    GoalCompleted { goal: GoalId },
    EntityState { entity: EntityId, state: String },
    Observation { description: String },
    Time { after: chrono::DateTime<chrono::Utc> },
}
