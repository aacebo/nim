use async_nats::jetstream;

use crate::{Error, Signal};

pub struct Stream<T> {
    inner: jetstream::consumer::pull::Stream,
    __data__: std::marker::PhantomData<T>,
}

impl<T> From<jetstream::consumer::pull::Stream> for Stream<T> {
    fn from(inner: jetstream::consumer::pull::Stream) -> Self {
        Self {
            inner,
            __data__: std::marker::PhantomData,
        }
    }
}

impl<T> Unpin for Stream<T> {}

impl<T> futures_util::Stream for Stream<T>
where
    T: for<'a> serde::Deserialize<'a>,
{
    type Item = Result<Signal<T>, Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let pin = std::pin::pin!(&mut self.inner);
        let next = futures_util::Stream::poll_next(pin, cx);

        next.map(|v| {
            let res = v?;

            let message = match res {
                Err(err) => return Some(Err(Error::Nats(err.into()))),
                Ok(v) => v,
            };

            let signal: Signal<T> = match serde_json::from_slice(&message.payload) {
                Err(err) => return Some(Err(err.into())),
                Ok(v) => v,
            };

            Some(Ok(signal))
        })
    }
}
