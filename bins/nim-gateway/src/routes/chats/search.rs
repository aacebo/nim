use axum::response::IntoResponse;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Query {
    #[validate(min_length = 1)]
    #[serde(rename = "q")]
    pub text: String,
}

pub async fn route(ctx: HttpContext, query: axum::extract::Query<Query>) -> axum::response::Result<axum::response::Response> {
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .embed(
            &[&query.text],
            nim::runtime::ai::TextEmbedOptions {
                model: nim::ai::pipelines::text::defaults::embed(),
                task: Default::default(),
            },
        )
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    let Some(artifact) = artifacts.first() else {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into());
    };

    let Some(embedding) = &artifact.vector else {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into());
    };

    let items = ctx
        .services()
        .chats()
        .search(
            embedding,
            nim::store::ChatFilter {
                limit: Some(10),
                min_score: Some(0.5),
            },
        )
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    Ok(axum::Json(items).into_response())
}
