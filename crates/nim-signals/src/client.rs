use async_nats::{jetstream, service::ServiceExt};

use crate::{Error, Service, ServiceError, Signal, Stream};

#[derive(Clone)]
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

    pub fn source(&self, source_id: uuid::Uuid) -> SourceClient {
        SourceClient {
            source_id,
            inner: self.clone(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn service(&self, name: &str, version: &str) -> Result<Service, Error> {
        let service = self.context.client().service_builder().start(name, version).await?;
        Ok(service.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn publish<S, T>(&self, subject: S, signal: T) -> Result<(), Error>
    where
        S: std::fmt::Debug + async_nats::ToSubject,
        T: std::fmt::Debug + serde::Serialize,
    {
        let signal = Signal {
            id: uuid::Uuid::now_v7(),
            source_id: None,
            content: signal,
            sent_at: chrono::Utc::now(),
        };

        let bytes = serde_json::to_vec(&signal)?;
        let _ = self.context.publish(subject, bytes.into()).await?.await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn subscribe<S, T>(&self, subject: S) -> Result<Stream<T>, Error>
    where
        S: std::fmt::Debug + async_nats::ToSubject,
    {
        let name = self.context.stream_by_subject(subject.to_subject().as_str()).await?;
        let stream = self.context.get_stream(&name).await?;

        let consumer = stream
            .get_or_create_consumer(
                &self.name,
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.name.clone()),
                    filter_subject: subject.to_subject().into_string(),
                    ack_wait: std::time::Duration::from_mins(5),
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

        if let Some(headers) = &message.headers {
            if let Some(status) = headers.get(async_nats::service::NATS_SERVICE_ERROR) {
                let code = headers
                    .get(async_nats::service::NATS_SERVICE_ERROR_CODE)
                    .and_then(|value| value.as_str().parse().ok())
                    .unwrap_or_default();

                return Err(ServiceError {
                    status: status.as_str().to_string(),
                    code,
                }
                .into());
            }
        }

        Ok(serde_json::from_slice(&message.payload)?)
    }
}

pub struct SourceClient {
    source_id: uuid::Uuid,
    inner: Client,
}

impl SourceClient {
    #[tracing::instrument(skip(self))]
    pub async fn publish<S, T>(&self, subject: S, signal: T) -> Result<(), Error>
    where
        S: std::fmt::Debug + async_nats::ToSubject,
        T: std::fmt::Debug + serde::Serialize,
    {
        let signal = Signal {
            id: uuid::Uuid::now_v7(),
            source_id: Some(self.source_id),
            content: signal,
            sent_at: chrono::Utc::now(),
        };

        let bytes = serde_json::to_vec(&signal)?;
        let _ = self.inner.context.publish(subject, bytes.into()).await?.await?;
        Ok(())
    }
}
