use std::os::unix::fs::MetadataExt;

use axum::response::IntoResponse;
use tokio::io::AsyncWriteExt;

use crate::http::HttpContext;

pub async fn route(
    ctx: HttpContext,
    mut multipart: axum::extract::Multipart,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    let mut artifacts = vec![];

    while let Some(mut field) = multipart.next_field().await? {
        let content_type = field.content_type();
        let mime_type = content_type.and_then(|v| v.parse::<mime::Mime>().ok());
        let ext = field
            .file_name()
            .and_then(|name| {
                std::path::Path::new(name)
                    .extension()
                    .filter(|ext| !ext.is_empty())
                    .map(|ext| ext.to_string_lossy().into_owned())
            })
            .or_else(|| {
                content_type
                    .and_then(mime_guess::get_mime_extensions_str)
                    .and_then(|exts| exts.first())
                    .map(|ext| ext.to_string())
            });

        let Some(ext) = ext else {
            return Ok((
                axum::http::StatusCode::BAD_REQUEST,
                "file extension or content type are needed to infer file extension",
            )
                .into_response());
        };

        let name = field
            .file_name()
            .map(String::from)
            .unwrap_or(format!("{}.{}", uuid::Uuid::now_v7().simple(), ext));
        let subdir = content_type.and_then(|content_type| match content_type {
            v if v.starts_with("audio/") => Some("audio"),
            v if v.starts_with("image/") => Some("images"),
            v if v.starts_with("text/") => Some("text"),
            _ => None,
        });

        let mut path = ctx.staging_dir.clone();

        if let Some(subdir) = subdir {
            path = path.join(subdir);
        }

        path = path.join(format!("{}.{}", uuid::Uuid::now_v7().simple(), ext));

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = tokio::fs::File::create(&path).await?;

        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
        }

        let size = file.metadata().await?.size();

        let artifact = ctx
            .services()
            .artifacts()
            .create(nim::state::ArtifactCreate {
                name,
                content: nim::state::ArtifactContent::File {
                    path,
                    size,
                    media_type: mime_type.clone().map(|v| v.to_string()),
                },
                summary: None,
                embedding: None,
                confidence: None,
                salience: None,
                strength: None,
            })
            .await?;

        let media_path = mime_type
            .map(|v| match v.type_() {
                mime::AUDIO => "audio",
                mime::IMAGE => "vision",
                mime::VIDEO => "vision",
                mime::TEXT => "text",
                _ => "unknown",
            })
            .unwrap_or("unknown");

        ctx.signals()
            .publish(format!("nim.{media_path}.artifact.create"), artifact.clone())
            .await?;

        artifacts.push(artifact);
    }

    Ok((axum::http::StatusCode::CREATED, axum::Json(artifacts)).into_response())
}
