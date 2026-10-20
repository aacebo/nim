use std::str::FromStr;

use axum::response::IntoResponse;
use tokio::io::AsyncWriteExt;

use crate::http::HttpContext;

pub async fn route(
    _ctx: HttpContext,
    mut multipart: axum::extract::Multipart,
) -> axum::response::Result<axum::response::Response> {
    let staging_dir = std::env::var("STAGING_DIR").map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let staging_dir = std::path::PathBuf::from_str(&staging_dir)?;

    while let Some(mut field) = multipart.next_field().await? {
        let content_type = field.content_type();
        let _mime_type = content_type.and_then(|v| v.parse::<mime::Mime>().ok());
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

        let mut path = staging_dir.clone();

        if let Some(subdir) = subdir {
            path = path.join(subdir);
        }

        path = path.join(&name);

        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk)
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok(axum::http::StatusCode::OK.into_response())
}
