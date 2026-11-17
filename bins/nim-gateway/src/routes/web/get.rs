use axum::{
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};

use super::views;

pub async fn route() -> nim::error::Result<impl axum::response::IntoResponse> {
    Ok(match views::render() {
        Ok(body) => (StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    })
}
