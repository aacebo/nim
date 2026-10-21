use async_nats::service;

use crate::{Error, ServiceError, Signal};

pub struct Request<T> {
    signal: Signal<T>,
    inner: service::Request,
}

impl<T> Request<T> {
    pub(crate) fn new(signal: Signal<T>, inner: service::Request) -> Self {
        Self { signal, inner }
    }

    #[tracing::instrument(skip(self))]
    pub async fn respond<R>(&self, response: Result<R, ServiceError>) -> Result<(), Error>
    where
        R: std::fmt::Debug + serde::Serialize,
    {
        let response = match response {
            Ok(content) => Ok(serde_json::to_vec(&Signal::from(content))?.into()),
            Err(err) => Err(err),
        };

        self.inner.respond(response).await?;
        Ok(())
    }
}

impl<T> std::ops::Deref for Request<T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}
