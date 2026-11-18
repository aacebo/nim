use crate::types::Span;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    Citation(Citation),
    Classification(Classification),
}

impl From<Citation> for Annotation {
    fn from(value: Citation) -> Self {
        Self::Citation(value)
    }
}

impl From<Classification> for Annotation {
    fn from(value: Classification) -> Self {
        Self::Classification(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Classification {
    pub name: String,
    pub label: String,
    pub score: f64,
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Citation {
    pub id: String,
    pub span: Span,
    pub url: Option<url::Url>,
    pub title: Option<String>,
    pub description: Option<String>,
}
