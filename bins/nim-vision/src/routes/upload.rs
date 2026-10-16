use axum::response::IntoResponse;

use crate::State;

pub async fn route(_state: State, mut multipart: axum::extract::Multipart) -> axum::response::Result<axum::response::Response> {
    let mut files = vec![];

    while let Some(field) = multipart.next_field().await? {
        let content_type = field.content_type().unwrap_or("application/octet-stream");

        if !content_type.starts_with("image/") {
            return Err((axum::http::StatusCode::BAD_REQUEST, "File must be an image".to_string()).into());
        }

        let bytes = field.bytes().await?;
        files.push(bytes.to_vec());
    }

    Ok(axum::http::StatusCode::OK.into_response())
}
