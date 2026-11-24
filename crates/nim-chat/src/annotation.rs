use std::collections::BTreeMap;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    Citation(Citation),
    Classification(Classification),
}

impl Annotation {
    pub fn start(&self) -> usize {
        match self {
            Self::Citation(v) => v.start,
            Self::Classification(v) => v.start,
        }
    }

    pub fn end(&self) -> usize {
        match self {
            Self::Citation(v) => v.end,
            Self::Classification(v) => v.end,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Citation(v) => v.description.as_deref(),
            Self::Classification(v) => v.description.as_deref(),
        }
    }

    pub fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        match self {
            Self::Citation(v) => &v.extra,
            Self::Classification(v) => &v.extra,
        }
    }
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
pub struct Citation {
    pub url: Option<url::Url>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub start: usize,
    pub end: usize,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Classification {
    pub label: String,
    pub description: Option<String>,
    pub confidence: f64,
    pub start: usize,
    pub end: usize,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
