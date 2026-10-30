#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArtifactRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub name: String,
    pub summary: Option<String>,
    pub embedding: Option<pgvector::Vector>,
    pub confidence: f32,
    pub salience: f32,
    pub strength: f32,
    pub content: sqlx::types::Json<ArtifactContent>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryArtifactRow {
    pub memory_id: uuid::Uuid,
    pub artifact_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactContent {
    Text {
        text: String,
    },
    File {
        path: std::path::PathBuf,
        size: u64,
        media_type: Option<String>,
    },
}

impl ArtifactContent {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text { text: value.into() }
    }

    pub fn file(path: impl Into<std::path::PathBuf>, size: u64, media_type: Option<String>) -> Self {
        Self::File {
            path: path.into(),
            size,
            media_type,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::File { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            Self::File { path, .. } => path.to_str(),
        }
    }
}

impl std::fmt::Display for ArtifactContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text { text } => write!(f, "{text}"),
            Self::File { path, .. } => write!(f, "{}", path.display()),
        }
    }
}
