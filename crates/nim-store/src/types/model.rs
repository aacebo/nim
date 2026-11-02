#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModelRow {
    pub id: uuid::Uuid,
    pub version: i32,
    #[sqlx(rename = "type")]
    pub ty: String,
    pub name: String,
    pub tasks: Vec<String>,
    pub architectures: Vec<String>,
    pub source: sqlx::types::Json<ModelSource>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ModelSource {
    Local(LocalModelSource),
    Remote {
        provider: String,
        id: String,
        base_url: Option<String>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalModelSource {
    Hub {
        id: String,
        #[serde(default)]
        resource: Option<ModelResource>,
    },
    Path {
        uri: String,
        format: String,
        path: Option<std::path::PathBuf>,
        size: u64,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelResource {
    pub uri: String,
    pub format: String,
    pub path: Option<std::path::PathBuf>,
    pub size: u64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModelWeightRow {
    pub id: uuid::Uuid,
    pub model_id: uuid::Uuid,
    pub version: i32,
    pub name: String,
    pub content: sqlx::types::JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
