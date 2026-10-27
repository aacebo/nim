use crate::{Context, Error};

#[derive(Clone)]
pub struct SignalContext {
    inner: Context,
    message: nim::signals::Message,
    signals: std::sync::Arc<nim::signals::SourceClient>,
}

impl SignalContext {
    pub(crate) fn new(inner: Context, source_id: uuid::Uuid, message: nim::signals::Message) -> Self {
        let signals = inner.signals.source(source_id);

        Self {
            inner,
            message,
            signals: std::sync::Arc::new(signals),
        }
    }

    pub fn span(&self) -> &tracing::Span {
        &self.inner.span
    }

    pub fn services(&self) -> nim::state::Services<'_> {
        self.inner.services()
    }

    pub fn signals(&self) -> &nim::signals::SourceClient {
        &self.signals
    }

    #[tracing::instrument(skip(self))]
    pub async fn ack(&self) -> Result<(), Error> {
        Ok(self.message.ack().await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn ack_progress(&self) -> Result<(), Error> {
        Ok(self.message.ack_with(nim::signals::AckKind::Progress).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn nack(&self) -> Result<(), Error> {
        Ok(self
            .message
            .ack_with(nim::signals::AckKind::Nak(Some(std::time::Duration::from_millis(1000 * 5))))
            .await?)
    }
}
