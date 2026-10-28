use async_nats::jetstream;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{Error, Signal, Stream, headers::NatsInjector};

#[derive(Clone)]
pub struct Client {
    name: String,
    context: jetstream::Context,
    propagator: TraceContextPropagator,
}

impl Client {
    pub(crate) fn new(name: impl AsRef<str>, context: jetstream::Context) -> Self {
        Self {
            name: name.as_ref().to_string(),
            context,
            propagator: TraceContextPropagator::new(),
        }
    }

    pub fn source(&self, source_id: uuid::Uuid) -> SourceClient {
        SourceClient {
            source_id,
            inner: self.clone(),
        }
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
        let context = tracing::Span::current().context();
        let mut headers = async_nats::HeaderMap::new();
        self.propagator
            .inject_context(&context, &mut NatsInjector::from(&mut headers));
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
        Ok(Stream::new(stream, self.propagator.clone()))
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
        let context = tracing::Span::current().context();
        let signal = Signal {
            id: uuid::Uuid::now_v7(),
            source_id: Some(self.source_id),
            content: signal,
            sent_at: chrono::Utc::now(),
        };

        let bytes = serde_json::to_vec(&signal)?;
        let mut headers = async_nats::HeaderMap::new();
        self.inner
            .propagator
            .inject_context(&context, &mut NatsInjector::from(&mut headers));
        let _ = self
            .inner
            .context
            .publish_with_headers(subject, headers, bytes.into())
            .await?
            .await?;
        Ok(())
    }
}
