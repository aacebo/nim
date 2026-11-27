#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: String,
    pub summary: Option<String>,
    pub mime_type: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Media {
    Path {
        path: std::path::PathBuf,
        name: Option<String>,
        size: Option<usize>,
        mime_type: Option<String>,
        summary: Option<String>,
    },
    Url {
        url: url::Url,
        name: Option<String>,
        size: Option<usize>,
        mime_type: Option<String>,
        summary: Option<String>,
    },
    Data {
        data: Vec<u8>,
        name: Option<String>,
        mime_type: String,
        summary: Option<String>,
    },
}

impl Media {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Path { name, .. } => name.as_deref(),
            Self::Url { name, .. } => name.as_deref(),
            Self::Data { name, .. } => name.as_deref(),
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Path { size, .. } => size.as_ref().map(|v| *v),
            Self::Url { size, .. } => size.as_ref().map(|v| *v),
            Self::Data { data, .. } => Some(data.len()),
        }
    }

    pub fn mime_type(&self) -> Option<&str> {
        match self {
            Self::Path { mime_type, .. } => mime_type.as_deref(),
            Self::Url { mime_type, .. } => mime_type.as_deref(),
            Self::Data { mime_type, .. } => Some(mime_type),
        }
    }

    pub fn summary(&self) -> Option<&str> {
        match self {
            Self::Path { summary, .. } => summary.as_deref(),
            Self::Url { summary, .. } => summary.as_deref(),
            Self::Data { summary, .. } => summary.as_deref(),
        }
    }
}
