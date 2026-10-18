use axum::response::IntoResponse;

use crate::http::HttpContext;

pub async fn route(
    _ctx: HttpContext,
    mut multipart: axum::extract::Multipart,
) -> axum::response::Result<axum::response::Response> {
    let mut audio = vec![];

    while let Some(field) = multipart.next_field().await? {
        let content_type = field.content_type().unwrap_or("application/octet-stream");

        if !content_type.starts_with("audio/") {
            return Err((axum::http::StatusCode::BAD_REQUEST, "File must be audio".to_string()).into());
        }

        let bytes = field.bytes().await?;
        audio.push(bytes.to_vec());
    }

    let artifacts = nim::ai::pipelines::audio_to_text(nim::ai::pipelines::AudioToTextArgs {
        audio,
        ..Default::default()
    })
    .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(axum::Json(artifacts).into_response())
}
