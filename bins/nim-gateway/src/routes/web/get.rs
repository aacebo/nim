use askama::Template;
use axum::{
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};

use super::views;
use crate::http::HttpContext;

pub async fn route(ctx: HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    let view = views::root::RootView::load(&ctx).await?;

    Ok(match view.render() {
        Ok(body) => (StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    })
}
