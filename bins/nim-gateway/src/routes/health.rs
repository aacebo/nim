use crate::http::HttpContext;

pub async fn route(_ctx: HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    Ok(axum::http::StatusCode::OK)
}
