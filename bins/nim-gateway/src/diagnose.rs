use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    handler::Handler,
    response::{IntoResponse, Response},
};
use std::{future::Future, pin::Pin};

#[derive(Clone)]
pub struct Diagnose<F>(F);

pub fn diagnose<F>(handler: F) -> Diagnose<F> {
    Diagnose(handler)
}

fn diagnose_error(error: nim::error::Diagnostic) -> Response {
    if let nim::error::Source::Error(source) = error.source() {
        if let Some(errors) = source.downcast_ref::<serde_valid::validation::Errors>() {
            return (axum::http::StatusCode::BAD_REQUEST, axum::Json(errors)).into_response();
        }
    }

    let status = match error.source() {
        nim::error::Source::Error(source)
            if source
                .downcast_ref::<sqlx::Error>()
                .is_some_and(|err| matches!(err, sqlx::Error::RowNotFound)) =>
        {
            axum::http::StatusCode::NOT_FOUND
        }
        nim::error::Source::Error(source)
            if source.downcast_ref::<axum::extract::multipart::MultipartError>().is_some()
                || source.downcast_ref::<axum::extract::rejection::JsonRejection>().is_some() =>
        {
            axum::http::StatusCode::BAD_REQUEST
        }
        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, error.to_string()).into_response()
}

impl<F, Fut, T, S> Handler<((),), S> for Diagnose<F>
where
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = nim::error::Result<T>> + Send + 'static,
    T: IntoResponse,
    S: Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, _req: Request, _state: S) -> Self::Future {
        Box::pin(async move {
            match (self.0)().await {
                Ok(response) => response.into_response(),
                Err(error) => diagnose_error(error),
            }
        })
    }
}

macro_rules! impl_handler {
    ([$($ty:ident),*], $last:ident) => {
        impl<F, Fut, S, T, M, $($ty,)* $last> Handler<(M, $($ty,)* $last,), S> for Diagnose<F>
        where
            F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = nim::error::Result<T>> + Send + 'static,
            S: Send + Sync + 'static,
            T: IntoResponse,
            $($ty: FromRequestParts<S> + Send,)*
            $last: FromRequest<S, M> + Send,
        {
            type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

            #[allow(non_snake_case)]
            fn call(self, req: Request, state: S) -> Self::Future {
                #[allow(unused_mut)]
                let (mut parts, body) = req.into_parts();

                Box::pin(async move {
                    $(
                        let $ty = match $ty::from_request_parts(&mut parts, &state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let req = Request::from_parts(parts, body);
                    let $last = match $last::from_request(req, &state).await {
                        Ok(value) => value,
                        Err(rejection) => return rejection.into_response(),
                    };

                    match (self.0)($($ty,)* $last,).await {
                        Ok(response) => response.into_response(),
                        Err(error) => diagnose_error(error),
                    }
                })
            }
        }
    };
}

impl_handler!([], T1);
impl_handler!([T1], T2);
impl_handler!([T1, T2], T3);
