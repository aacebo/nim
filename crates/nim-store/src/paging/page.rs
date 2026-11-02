#[derive(Debug, Clone)]
pub struct Page {
    pub cursor: Option<(uuid::Uuid, chrono::DateTime<chrono::Utc>)>,
    pub limit: usize,
}
