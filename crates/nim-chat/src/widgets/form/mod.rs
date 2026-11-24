mod input;
mod select;

pub use input::*;
pub use select::*;

use crate::widgets::{Action, Element, ElementId};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Form {
    #[serde(default)]
    pub id: ElementId,
    pub body: Vec<Element>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submit: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<Action>,
}
