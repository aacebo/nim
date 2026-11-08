use axum::response::IntoResponse;

use crate::{Error, extract, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Body {
    pub name: String,
    pub text: String,
}

pub async fn route(ctx: HttpContext, body: extract::Json<Body>) -> axum::response::Result<axum::response::Response> {
    let artifact = ctx
        .services()
        .artifacts()
        .create(nim::state::ArtifactCreate {
            name: body.name.clone(),
            content: nim::state::ArtifactContent::Text { text: body.text.clone() },
            summary: None,
            embedding: None,
            confidence: None,
            salience: None,
            strength: None,
        })
        .await
        .map_err(Error::from)?;

    ctx.signals()
        .publish("nim.text.artifact.create", artifact.clone())
        .await
        .map_err(Error::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(artifact)).into_response())
}
