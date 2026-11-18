use crate::types::Span;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Citation {
    pub id: String,
    pub span: Span,
    pub url: url::Url,
    pub title: Option<String>,
    pub description: Option<String>,
}
