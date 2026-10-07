use crate::EntityId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub name: ActionName,
    pub target: Option<EntityId>,
    pub message: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionName {
    Say,
    Do,
    Recall,
}
