use std::sync::Arc;
use std::time::Instant;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;

use crate::Error;

const REQUEST_ID_HEADER: &str = "X-Request-ID";

#[derive(Clone)]
pub struct HttpContext {
    #[allow(unused)]
    request_id: uuid::Uuid,
    state: Arc<super::Context>,
    span: tracing::Span,
}

impl HttpContext {
    pub fn span(&self) -> &tracing::Span {
        &self.span
    }
}

impl std::ops::Deref for HttpContext {
    type Target = super::Context;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<S> FromRequestParts<S> for HttpContext
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<HttpContext>()
            .cloned()
            .expect("HttpContext not found in request extensions");

        Ok(ctx)
    }
}

#[tracing::instrument(
    level = "info",
    name = "http.request",
    skip_all,
    fields(
        request_id = tracing::field::Empty,
        method = tracing::field::Empty,
        path = tracing::field::Empty,
        status = tracing::field::Empty,
        elapsed_ms = tracing::field::Empty,
    )
)]
pub async fn middleware(State(state): State<Arc<super::Context>>, mut request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let headers = request.headers().clone();
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(uuid::Uuid::now_v7);

    let span = tracing::Span::current();
    span.record("request_id", tracing::field::display(request_id));
    span.record("method", tracing::field::display(&method));
    span.record("path", tracing::field::display(&path));

    request.extensions_mut().insert(HttpContext {
        state,
        request_id,
        span: span.clone(),
    });

    let started_at = Instant::now();
    tracing::debug!("request started");
    let response = next.run(request).await;
    let elapsed_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    span.record("status", status);
    span.record("elapsed_ms", elapsed_ms);

    if status >= 500 {
        tracing::error!("request completed");
    } else {
        tracing::info!("request completed");
    }

    response
}
