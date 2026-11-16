use axum::response::IntoResponse;

use crate::{extract, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Body {
    pub name: String,
    pub text: String,
}

pub async fn route(ctx: HttpContext, body: extract::Json<Body>) -> nim::error::Result<impl axum::response::IntoResponse> {
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
        .await?;

    ctx.signals().publish("nim.text.artifact.create", artifact.clone()).await?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(artifact)).into_response())
}
