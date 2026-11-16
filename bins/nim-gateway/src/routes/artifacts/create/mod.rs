mod files;
mod json;

use axum::{RequestExt, response::IntoResponse};

use crate::http::HttpContext;

pub async fn route(ctx: HttpContext, req: axum::extract::Request) -> nim::error::Result<impl axum::response::IntoResponse> {
    let content_type = req
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    match content_type {
        Some("application/json") => {
            let body: crate::extract::Json<json::Body> = match req.extract().await {
                Ok(body) => body,
                Err(response) => return Ok(axum::response::IntoResponse::into_response(response)),
            };
            Ok(json::route(ctx, body).await?.into_response())
        }
        Some(v) if v.starts_with("multipart/form-data") => {
            let multipart: axum::extract::Multipart = match req.extract().await {
                Ok(multipart) => multipart,
                Err(response) => return Ok(axum::response::IntoResponse::into_response(response)),
            };
            Ok(files::route(ctx, multipart).await?.into_response())
        }
        Some(v) => Ok((axum::http::StatusCode::BAD_REQUEST, format!("invalid Content-Type `{v}`")).into_response()),
        None => Ok((axum::http::StatusCode::BAD_REQUEST, "missing Content-Type").into_response()),
    }
}
