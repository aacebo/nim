use async_nats::service;

use crate::{Endpoint, Error};

#[derive(Debug)]
pub struct Service {
    inner: service::Service,
}

impl Service {
    #[tracing::instrument(skip(self))]
    pub async fn endpoint<S>(&self, subject: S) -> Result<Endpoint, Error>
    where
        S: std::fmt::Debug + ToString,
    {
        Ok(self.inner.endpoint(subject).await?.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn stop(self) -> Result<(), Error> {
        self.inner.stop().await?;
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
