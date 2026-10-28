use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};

pub fn layer<S>(name: impl std::fmt::Display) -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(std::env::var("ALLOY_URL").unwrap_or("http://alloy:4317".into()))
        .build()
        .unwrap();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name(name.to_string()).build())
        .build();

    let tracer = provider.tracer(name.to_string());
    tracing_opentelemetry::layer().with_tracer(tracer)
}
