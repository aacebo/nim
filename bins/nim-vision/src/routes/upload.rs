use axum::response::IntoResponse;

use crate::State;

pub async fn route(_state: State, _multipart: axum::extract::Multipart) -> axum::response::Response {
    axum::http::StatusCode::OK.into_response()
}
