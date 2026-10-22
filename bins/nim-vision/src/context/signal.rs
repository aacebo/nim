use crate::{Context, Error};

#[derive(Clone)]
pub struct SignalContext {
    inner: Context,
    message: nim::signals::Message,
}

impl SignalContext {
    pub(crate) fn new(inner: Context, message: nim::signals::Message) -> Self {
        Self { inner, message }
    }

    pub fn span(&self) -> &tracing::Span {
        &self.span
    }

    #[tracing::instrument(skip(self))]
    pub async fn ack(&self) -> Result<(), Error> {
        Ok(self.message.ack().await?)
    }
}

impl std::ops::Deref for SignalContext {
    type Target = super::Context;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
