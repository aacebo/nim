use async_nats::service;

use crate::Endpoint;

#[derive(Debug)]
pub struct Service {
    inner: service::Service,
}

impl Service {
    #[tracing::instrument(skip(self))]
    pub async fn endpoint<S>(&self, subject: S) -> nim_error::Result<Endpoint>
    where
        S: std::fmt::Debug + ToString,
    {
        Ok(self
            .inner
            .endpoint(subject)
            .await
            .map_err(nim_error::Diagnostic::message)?
            .into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn stop(self) -> nim_error::Result<()> {
        self.inner.stop().await.map_err(nim_error::Diagnostic::message)?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn reset(&mut self) {
        self.inner.reset().await;
    }
}

impl From<service::Service> for Service {
    fn from(inner: service::Service) -> Self {
        Self { inner }
    }
}
