use async_nats::jetstream;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{Error, RawSignal, headers::NatsExtractor};

pub struct Stream {
    inner: jetstream::consumer::pull::Stream,
    propagator: TraceContextPropagator,
}

impl Stream {
    pub(crate) fn new(inner: jetstream::consumer::pull::Stream, propagator: TraceContextPropagator) -> Self {
        Self { inner, propagator }
    }
}

impl Unpin for Stream {}

impl futures_util::Stream for Stream {
    type Item = Result<(jetstream::Message, RawSignal, tracing::Span), Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let pin = std::pin::pin!(&mut self.inner);
        let next = futures_util::Stream::poll_next(pin, ctx);

        next.map(|v| {
            let res = v?;

            let message = match res {
                Err(err) => return Some(Err(Error::Nats(err.into()))),
                Ok(v) => v,
            };

            let signal: RawSignal = match serde_json::from_slice(&message.payload) {
                Err(err) => return Some(Err(err.into())),
                Ok(v) => v,
            };

            let span = tracing::info_span!(
                parent: None,
                "signal.process",
                otel.kind = "consumer",
                otel.status_code = tracing::field::Empty,
                messaging.system = "nats",
                messaging.destination.name = %message.subject,
                nim.signal.source = tracing::field::Empty,
            );

            if let Some(headers) = &message.headers {
                if let Some(source) = headers.get("nim-source-service") {
                    span.record("nim.signal.source", source.as_str());
                }

                let parent = self.propagator.extract(&NatsExtractor::from(headers));

                match span.set_parent(parent) {
                    Err(err) => return Some(Err(err.into())),
                    Ok(v) => v,
                };
            }

            Some(Ok((message, signal, span)))
        })
    }
}
