use axum::response::IntoResponse;

use crate::http::HttpContext;

pub async fn route(_ctx: HttpContext) -> axum::response::Result<axum::response::Response> {
    Ok(axum::http::StatusCode::OK.into_response())
}
