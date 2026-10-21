use async_nats::jetstream;

use crate::{Error, Signal, Stream};

pub struct Client {
    name: String,
    context: jetstream::Context,
}

impl Client {
    pub(crate) fn new(name: impl AsRef<str>, context: jetstream::Context) -> Self {
        Self {
            name: name.as_ref().to_string(),
            context,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn publish<S, T>(&self, subject: S, signal: T) -> Result<(), Error>
    where
        S: std::fmt::Debug + async_nats::ToSubject,
        T: std::fmt::Debug + serde::Serialize,
    {
        let signal = Signal::from(signal);
        let bytes = serde_json::to_vec(&signal)?;
        let _ = self.context.publish(subject, bytes.into()).await?.await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn subscribe<S, T>(&self, subject: S) -> Result<Stream<Signal<T>>, Error>
    where
        S: std::fmt::Debug + async_nats::ToSubject,
    {
        let name = self.context.stream_by_subject(subject.to_subject().as_str()).await?;
        let stream = self
            .context
            .get_stream(&name)
            .await?;

        let consumer = stream
            .get_or_create_consumer(
                &self.name,
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.name.clone()),
                    ..Default::default()
                },
            )
            .await?;

        let stream = consumer.messages().await?;
        Ok(stream.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn request<S, T, R>(&self, subject: S, signal: T) -> Result<Signal<R>, Error>
    where
        S: std::fmt::Debug + async_nats::ToSubject,
        T: std::fmt::Debug + serde::Serialize,
        R: std::fmt::Debug + for<'a> serde::Deserialize<'a>,
    {
        let signal: Signal<T> = signal.into();
        let signal = serde_json::to_vec(&signal)?;
        let message = self.context.client().request(subject, signal.into()).await?;
        Ok(serde_json::from_slice(&message.payload)?)
    }
}
