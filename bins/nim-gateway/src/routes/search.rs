use axum::response::IntoResponse;

use crate::{Error, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Query {
    #[validate(min_length = 1)]
    #[serde(rename = "q")]
    pub text: String,

    #[validate(minimum = 5)]
    #[validate(maximum = 500)]
    #[serde(default = "defaults::limit")]
    pub limit: usize,

    #[validate(minimum = 0.0)]
    #[validate(maximum = 1.0)]
    #[serde(default = "defaults::min_score")]
    pub min_score: f32,
}

pub async fn route(ctx: HttpContext, query: axum::extract::Query<Query>) -> axum::response::Result<axum::response::Response> {
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .embed(
            &[&query.text],
            nim::runtime::ai::TextEmbedOptions {
                embedding: nim::ai::models::EmbedOptions {
                    mode: nim::ai::models::EmbedMode::Query,
                    dimensions: Some(768),
                    ..Default::default()
                },
                model: nim::ai::models::refer("unsloth/embeddinggemma-300m").map_err(Error::from)?,
                task: Default::default(),
            },
        )
        .await
        .map_err(Error::from)?;

    let Some(artifact) = artifacts.first() else {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into());
    };

    let Some(embedding) = &artifact.vector else {
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into());
    };

    let items = ctx
        .services()
        .search()
        .all(
            embedding,
            nim::store::Filter {
                limit: Some(query.limit),
                min_score: Some(query.min_score),
            },
        )
        .await
        .map_err(Error::from)?;

    Ok(axum::Json(items).into_response())
}

mod defaults {
    pub fn limit() -> usize {
        5
    }

    pub fn min_score() -> f32 {
        0.3
    }
}
