use axum::response::IntoResponse;

use crate::State;

pub async fn route(_state: State, mut multipart: axum::extract::Multipart) -> axum::response::Result<axum::response::Response> {
    while let Some(field) = multipart.next_field().await? {
        let content_type = field.content_type().unwrap_or("application/octet-stream");

        if !content_type.starts_with("audio/") {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "File must be an audio format".to_string(),
            )
                .into());
        }

        let _bytes = field.bytes().await?;
    }

    Ok(axum::http::StatusCode::OK.into_response())
}
