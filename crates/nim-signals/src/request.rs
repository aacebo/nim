use async_nats::{Message, service};

use crate::{Error, ServiceError, Signal};

pub struct Request {
    inner: service::Request,
}

impl Request {
    #[tracing::instrument(skip(self))]
    pub fn decode<T>(&self) -> Result<Signal<T>, Error>
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        let signal: Signal<T> = serde_json::from_slice(&self.payload)?;
        Ok(signal)
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

        Ok(self.inner.respond(response).await?)
    }
}

impl From<service::Request> for Request {
    fn from(inner: service::Request) -> Self {
        Self { inner }
    }
}

impl std::ops::Deref for Request {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.inner.message
    }
}
