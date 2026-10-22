pub trait ToSignal<T> {
    type Error;

    fn to_signal(&self) -> Result<Signal<T>, Self::Error>;
}

impl<T> ToSignal<T> for async_nats::Message
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Error = crate::Error;

    fn to_signal(&self) -> Result<Signal<T>, Self::Error> {
        Ok(serde_json::from_slice(&self.payload)?)
    }
}

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
