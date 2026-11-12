use axum::response::IntoResponse;
use tokio_util::io::ReaderStream;

use crate::{Error, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Params {
    pub artifact_id: nim::state::ArtifactId,
}

pub async fn route(ctx: HttpContext, params: axum::extract::Path<Params>) -> axum::response::Result<axum::response::Response> {
    let artifact = ctx
        .services()
        .artifacts()
        .find_by_id(params.artifact_id)
        .await
        .map_err(Error::from)?;

    match artifact.content {
        nim::state::ArtifactContent::Text { text } => Ok(text.into_response()),
        nim::state::ArtifactContent::File {
            path,
            size: _,
            media_type: _,
        } => {
            let file = match tokio::fs::File::open(&path).await {
                Ok(file) => file,
                Err(err) => return Err((axum::http::StatusCode::NOT_FOUND, format!("file not found: {}", err)).into()),
            };

            let stream = ReaderStream::new(file);
            let response = (
                [
                    (axum::http::header::CONTENT_TYPE, "application/octet-stream".into()),
                    (
                        axum::http::header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", artifact.name.escape_unicode()),
                    ),
                ],
                axum::body::Body::from_stream(stream),
            );

            Ok(response.into_response())
        }
    }
}
