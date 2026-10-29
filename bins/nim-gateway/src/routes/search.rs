use axum::response::IntoResponse;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Query {
    pub text: String,
}

pub async fn route(ctx: HttpContext, query: axum::extract::Query<Query>) -> axum::response::Result<axum::response::Response> {
    let artifacts = nim::ai::pipelines::text::embeddings::run(&[query.text.as_bytes()])
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    let Some(artifact) = artifacts.first() else {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into());
    };

    let Some(embedding) = &artifact.vector else {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into());
    };

    let results = ctx
        .services()
        .memories()
        .search(embedding.as_slice(), nim::state::MemoryType::Working, 10)
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    Ok(axum::Json(results).into_response())
}
