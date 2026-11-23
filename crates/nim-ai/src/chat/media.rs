#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Media {
    Path {
        name: Option<String>,
        path: std::path::PathBuf,
        size: Option<usize>,
        mime_type: Option<String>,
        summary: Option<String>,
    },
    Url {
        name: Option<String>,
        url: url::Url,
        size: Option<usize>,
        mime_type: Option<String>,
        summary: Option<String>,
    },
}

impl Media {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Path { name, .. } => name.as_deref(),
            Self::Url { name, .. } => name.as_deref(),
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Path { size, .. } => size.as_ref().map(|v| *v),
            Self::Url { size, .. } => size.as_ref().map(|v| *v),
        }
    }

    pub fn mime_type(&self) -> Option<&str> {
        match self {
            Self::Path { mime_type, .. } => mime_type.as_deref(),
            Self::Url { mime_type, .. } => mime_type.as_deref(),
        }
    }

    pub fn summary(&self) -> Option<&str> {
        match self {
            Self::Path { summary, .. } => summary.as_deref(),
            Self::Url { summary, .. } => summary.as_deref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub title: Option<String>,
    pub content: String,
    pub mime_type: Option<String>,
}
