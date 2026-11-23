use axum::response::IntoResponse;

use crate::{extract, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Body {
    pub name: String,
    pub text: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub headers: std::collections::BTreeMap<String, String>,
}

pub async fn route(ctx: HttpContext, body: extract::Json<Body>) -> nim::error::Result<impl axum::response::IntoResponse> {
    let Body {
        name,
        text,
        url,
        headers,
    } = body.into_inner();

    let content = match (text, url) {
        (Some(text), None) => nim::state::ArtifactContent::Text { text },
        (None, Some(url)) => {
            let url = url::Url::parse(&url)?;
            if !matches!(url.scheme(), "http" | "https") {
                return Ok((axum::http::StatusCode::BAD_REQUEST, "resource URL must use HTTP or HTTPS").into_response());
            }
            for (name, value) in &headers {
                name.parse::<axum::http::header::HeaderName>()?;
                value.parse::<axum::http::HeaderValue>()?;
            }
            nim::state::ArtifactContent::Resource {
                url: url.to_string(),
                headers,
                size: None,
                media_type: None,
            }
        }
        _ => {
            return Ok((axum::http::StatusCode::BAD_REQUEST, "provide exactly one of text or url").into_response());
        }
    };

    let resource = matches!(content, nim::state::ArtifactContent::Resource { .. });
    let artifact = ctx
        .services()
        .artifacts()
        .create(nim::state::ArtifactCreate {
            name,
            content,
            summary: None,
            embedding: None,
            confidence: None,
            salience: None,
            strength: None,
        })
        .await?;

    let subject = if resource {
        artifact.name.rsplit_once('.').and_then(|(_, extension)| {
            let mime = mime_guess::from_ext(extension).first_or_octet_stream();
            match mime.type_() {
                mime::AUDIO => Some("nim.audio.artifact.create"),
                mime::IMAGE | mime::VIDEO => Some("nim.vision.artifact.create"),
                mime::TEXT => Some("nim.text.artifact.create"),
                _ => None,
            }
        })
    } else {
        Some("nim.text.artifact.create")
    };

    if let Some(subject) = subject {
        ctx.signals().publish(subject, artifact.clone()).await?;
    }

    Ok((axum::http::StatusCode::CREATED, axum::Json(artifact)).into_response())
}
