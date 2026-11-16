use async_nats::service;

use crate::Request;

pub struct Endpoint {
    inner: service::endpoint::Endpoint,
}

impl Endpoint {
    #[tracing::instrument(skip(self))]
    pub async fn stop(&mut self) -> nim_error::Result<()> {
        self.inner.stop().await?;
        Ok(())
    }
}

impl From<service::endpoint::Endpoint> for Endpoint {
    fn from(inner: service::endpoint::Endpoint) -> Self {
        Self { inner }
    }
}

impl Unpin for Endpoint {}

impl futures_util::Stream for Endpoint {
    type Item = nim_error::Result<Request>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let pin = std::pin::pin!(&mut self.inner);
        let next = futures_util::Stream::poll_next(pin, ctx);
        next.map(|v| Some(Ok(v?.into())))
    }
}
