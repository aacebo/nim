#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Media {
    Path {
        path: std::path::PathBuf,
        summary: Option<String>,
    },
    Url {
        url: url::Url,
        summary: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub title: Option<String>,
    pub content: String,
    pub mime_type: Option<String>,
}
