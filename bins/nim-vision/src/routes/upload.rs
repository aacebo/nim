use axum::response::IntoResponse;

use crate::http::HttpContext;

pub async fn route(
    _ctx: HttpContext,
    mut multipart: axum::extract::Multipart,
) -> axum::response::Result<axum::response::Response> {
    let mut images = vec![];

    while let Some(field) = multipart.next_field().await? {
        let content_type = field.content_type().unwrap_or("application/octet-stream");

        if !content_type.starts_with("image/") {
            return Err((axum::http::StatusCode::BAD_REQUEST, "file must be an image".to_string()).into());
        }

        let bytes = field.bytes().await?;
        images.push(bytes.to_vec());
    }

    let artifacts = nim::ai::pipelines::images::captions::run(nim::ai::pipelines::images::captions::Args {
        images,
        ..Default::default()
    })
    .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(axum::Json(artifacts).into_response())
}
