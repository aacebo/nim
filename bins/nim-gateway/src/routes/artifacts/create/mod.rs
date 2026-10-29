mod files;
mod json;

use axum::RequestExt;

use crate::http::HttpContext;

pub async fn route(ctx: HttpContext, req: axum::extract::Request) -> axum::response::Result<axum::response::Response> {
    let content_type = req
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    match content_type {
        Some("application/json") => {
            let body = req.extract().await?;
            json::route(ctx, body).await
        }
        Some(v) if v.starts_with("multipart/form-data") => {
            let multipart = req.extract().await?;
            files::route(ctx, multipart).await
        }
        Some(v) => Err((axum::http::StatusCode::BAD_REQUEST, format!("invalid Content-Type `{v}`")).into()),
        None => Err((axum::http::StatusCode::BAD_REQUEST, "missing Content-Type").into()),
    }
}
