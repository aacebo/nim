use crate::Error;

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
    pub source_id: Option<uuid::Uuid>,
    pub content: T,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

impl<T> From<T> for Signal<T> {
    fn from(content: T) -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
            source_id: None,
            content,
            sent_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RawSignal {
    pub id: uuid::Uuid,
    pub source_id: Option<uuid::Uuid>,
    pub content: Box<serde_json::value::RawValue>,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

impl<T> ToSignal<T> for RawSignal
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Error = Error;

    fn to_signal(&self) -> Result<Signal<T>, Self::Error> {
        let content: T = serde_json::from_str(self.content.get())?;

        Ok(Signal {
            id: self.id,
            source_id: self.source_id,
            content,
            sent_at: self.sent_at,
        })
    }
}
