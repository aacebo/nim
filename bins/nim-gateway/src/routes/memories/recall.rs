use axum::response::IntoResponse;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub text: String,
}

pub async fn route(ctx: HttpContext, body: axum::extract::Json<Body>) -> axum::response::Result<axum::response::Response> {
    let artifacts = nim::ai::pipelines::text::embeddings::run(&[body.text.as_bytes()])
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
        .search(
            embedding.as_slice(),
            nim::store::MemoryFilter {
                limit: Some(10),
                min_score: Some(0.7),
                ..Default::default()
            },
        )
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    let recall = ctx
        .services()
        .recalls()
        .create(nim::state::RecallCreate {
            trigger: nim::state::RecallTrigger::Explicit,
            source: nim::state::RecallSource::User,
            query: Some(body.text.clone()),
        })
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    for item in &results {
        ctx.services()
            .recalls()
            .link(item.id, recall.id, item.similarity)
            .await
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;
    }

    ctx.signals()
        .publish("nim.recall.create", recall)
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    Ok(axum::Json(results).into_response())
}
