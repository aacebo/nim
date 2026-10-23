use std::{os::unix::fs::MetadataExt, str::FromStr};

use axum::response::IntoResponse;
use tokio::io::AsyncWriteExt;

use crate::http::HttpContext;

pub async fn route(
    ctx: HttpContext,
    mut multipart: axum::extract::Multipart,
) -> axum::response::Result<axum::response::Response> {
    let mut artifacts = vec![];

    while let Some(mut field) = multipart.next_field().await? {
        let content_type = field.content_type();
        let mime_type = content_type.and_then(|v| v.parse::<mime::Mime>().ok());
        let ext = if let Some(content_type) = &content_type {
            mime_guess::get_mime_extensions_str(content_type)
                .and_then(|exts| exts.first())
                .map(|ext| ext.to_string())
        } else if let Some(name) = field.file_name() {
            std::path::PathBuf::from_str(name)?
                .extension()
                .map(|ext| ext.display().to_string())
        } else {
            None
        };

        let Some(ext) = ext else {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "file extension or content type are needed to infer file extension",
            )
                .into());
        };

        let name = field
            .file_name()
            .map(String::from)
            .unwrap_or(format!("{}.{}", uuid::Uuid::now_v7().simple(), ext));
        let subdir = content_type.and_then(|content_type| match content_type {
            v if v.starts_with("audio/") => Some("audio"),
            v if v.starts_with("image/") => Some("images"),
            _ => None,
        });

        let mut path = ctx.staging_dir.clone();

        if let Some(subdir) = subdir {
            path = path.join(subdir);
        }

        path = path.join(&name);

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        }

        let mut file = tokio::fs::File::create(&path)
            .await
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk)
                .await
                .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        }

        let size = file
            .metadata()
            .await
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
            .size();

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
            })
            .await
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        let media_path = mime_type
            .map(|v| match v.type_() {
                mime::AUDIO => "audio",
                mime::IMAGE => "vision",
                mime::VIDEO => "vision",
                _ => "unknown",
            })
            .unwrap_or("unknown");

        ctx.signals()
            .publish(format!("nim.{media_path}.artifact.create"), artifact.clone())
            .await
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        artifacts.push(artifact);
    }

    Ok(axum::Json(artifacts).into_response())
}
