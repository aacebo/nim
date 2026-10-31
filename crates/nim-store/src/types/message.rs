#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageRow {
    pub id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub version: i32,
    pub role: Role,
    pub content: sqlx::types::Json<Vec<MessageContent>>,
    pub summary: Option<String>,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Role {
    System,
    Agent,
    User,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text(TextContent),
    File(FileContent),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum TextContent {
    String(String),
    Object { text: String },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum FileContent {
    Url { url: String },
    Data { data: String },
}
