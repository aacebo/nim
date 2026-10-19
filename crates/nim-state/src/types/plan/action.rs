pub use nim_store::ActionName;

use crate::EntityId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Action {
    pub name: ActionName,
    pub target: Option<EntityId>,
    pub message: String,
}
