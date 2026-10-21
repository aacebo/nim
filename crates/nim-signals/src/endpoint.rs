use async_nats::service;

use crate::{Error, Request, Signal};

pub struct Endpoint<T> {
    inner: service::endpoint::Endpoint,
    __data__: std::marker::PhantomData<T>,
}

impl<T> Endpoint<T> {
    #[tracing::instrument(skip(self))]
    pub async fn stop(&mut self) -> Result<(), Error> {
        self.inner.stop().await?;
        Ok(())
    }
}

impl<T> From<service::endpoint::Endpoint> for Endpoint<T> {
    fn from(inner: service::endpoint::Endpoint) -> Self {
        Self {
            inner,
            __data__: std::marker::PhantomData,
        }
    }
}

impl<T> Unpin for Endpoint<T> {}

impl<T> futures_core::Stream for Endpoint<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Item = Result<Request<T>, Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let pin = std::pin::pin!(&mut self.inner);
        let next = futures_core::Stream::poll_next(pin, cx);

        next.map(|v| {
            let request = v?;
            let signal: Signal<T> = match serde_json::from_slice(&request.message.payload) {
                Err(err) => return Some(Err(err.into())),
                Ok(v) => v,
            };

            Some(Ok(Request::new(signal, request)))
        })
    }
}
