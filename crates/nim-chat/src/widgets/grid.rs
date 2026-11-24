use crate::widgets::{Element, ElementId, Size};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Grid {
    #[serde(default)]
    pub id: ElementId,
    pub padding: Size,
    pub margin: Size,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Column {
    #[serde(default)]
    pub id: ElementId,
    pub padding: Size,
    pub margin: Size,
    pub body: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Row {
    #[serde(default)]
    pub id: ElementId,
    pub padding: Size,
    pub margin: Size,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colspan: Option<usize>,
    pub body: Vec<Element>,
}
