#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Signal<T> {
    pub id: uuid::Uuid,
    pub content: T,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

impl<T> From<T> for Signal<T> {
    fn from(content: T) -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
            content,
            sent_at: chrono::Utc::now(),
        }
    }
}
