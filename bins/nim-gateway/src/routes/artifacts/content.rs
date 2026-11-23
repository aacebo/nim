use axum::response::IntoResponse;
use tokio_util::io::ReaderStream;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Params {
    pub artifact_id: nim::state::ArtifactId,
}

pub async fn route(
    ctx: HttpContext,
    params: axum::extract::Path<Params>,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    let artifact = ctx.services().artifacts().find_by_id(params.artifact_id).await?;

    match artifact.content {
        nim::state::ArtifactContent::Text { text } => Ok(text.into_response()),
        nim::state::ArtifactContent::Resource { url, headers, .. } => {
            let client = reqwest::Client::new();
            let mut request = client.get(url);

            for (name, value) in headers {
                request = request.header(name, value);
            }

            let response = request.send().await?;
            let status = axum::http::StatusCode::from_u16(response.status().as_u16())?;
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let body = response.bytes().await?;
            Ok((status, [(axum::http::header::CONTENT_TYPE, content_type)], body).into_response())
        }
        nim::state::ArtifactContent::File {
            path,
            size: _,
            media_type: _,
        } => {
            let file = match tokio::fs::File::open(&path).await {
                Ok(file) => file,
                Err(err) => {
                    return Ok((axum::http::StatusCode::NOT_FOUND, format!("file not found: {}", err)).into_response());
                }
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
