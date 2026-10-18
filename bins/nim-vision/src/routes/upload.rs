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

    let artifacts = nim::ai::pipelines::images::captions::run(&images)
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    // let memory = ctx.services()
    //     .memories()
    //     .create(nim::state::MemoryCreate {
    //         ty: nim::state::MemoryType::Working,
    //         sources: &[nim::state::Source::Region(nim::state::Region::Vision)],
    //         contexts: &[],
    //         description: artifacts
    //             .,
    //     })
    //     .await?;

    Ok(axum::Json(artifacts).into_response())
}
