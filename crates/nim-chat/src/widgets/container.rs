use crate::widgets::{Element, ElementId, Size};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Container {
    #[serde(default)]
    pub id: ElementId,
    pub padding: Size,
    pub margin: Size,
    pub body: Vec<Element>,
}
