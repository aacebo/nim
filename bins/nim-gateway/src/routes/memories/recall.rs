use axum::response::IntoResponse;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub text: String,

    #[serde(default)]
    pub created_at: Option<nim::state::TimeRange>,
}

pub async fn route(ctx: HttpContext, body: axum::extract::Json<Body>) -> nim::error::Result<impl axum::response::IntoResponse> {
    let body = body.0;
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .embed(
            &[&body.text],
            nim::runtime::ai::TextEmbedOptions {
                embedding: nim::ai::EmbedOptions {
                    mode: nim::ai::EmbedMode::Query,
                    dimensions: Some(768),
                    ..Default::default()
                },
                model: nim::ai::models::refer("unsloth/embeddinggemma-300m")?,
                ..Default::default()
            },
        )
        .await?;

    let Some(artifact) = artifacts.first() else {
        return Ok(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };

    let Some(embedding) = &artifact.vector else {
        return Ok(axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response());
    };

    let results = ctx
        .services()
        .search()
        .memories(
            embedding.as_slice(),
            nim::store::MemoryFilter {
                limit: Some(10),
                min_score: Some(0.7),
                created_at: body.created_at.map(Into::into),
                ..Default::default()
            },
        )
        .await?;

    let recall = ctx
        .services()
        .recalls()
        .create(nim::state::RecallCreate {
            trigger: nim::state::RecallTrigger::Explicit,
            source: nim::state::RecallSource::User,
            query: Some(body.text.clone()),
        })
        .await?;

    for item in &results {
        let link = ctx.services().recalls().link(item.id, recall.id, item.score).await?;

        ctx.signals().publish("nim.memory.recall", link).await?;
    }

    Ok((axum::http::StatusCode::CREATED, axum::Json(results)).into_response())
}
