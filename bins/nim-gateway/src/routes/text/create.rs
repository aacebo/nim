use axum::response::IntoResponse;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub name: String,
    pub text: String,
}

pub async fn route(ctx: HttpContext, body: axum::Json<Body>) -> axum::response::Result<axum::response::Response> {
    let artifact = ctx
        .services()
        .artifacts()
        .create(nim::state::ArtifactCreate {
            name: body.name.clone(),
            content: nim::state::ArtifactContent::Text { text: body.text.clone() },
            summary: None,
            embedding: None,
        })
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    ctx.signals()
        .publish("nim.text.artifact.create", artifact.clone())
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(artifact)).into_response())
}
