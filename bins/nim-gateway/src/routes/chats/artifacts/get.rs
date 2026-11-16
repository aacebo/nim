use axum::response::IntoResponse;
use serde_valid::Validate;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Params {
    pub chat_id: nim::state::ChatId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Query {
    pub cursor: Option<nim::state::Cursor>,

    #[validate(minimum = 5)]
    #[validate(maximum = 500)]
    #[serde(default = "defaults::limit")]
    pub limit: usize,
}

pub async fn route(
    ctx: HttpContext,
    params: axum::extract::Path<Params>,
    query: axum::extract::Query<Query>,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    query.validate()?;
    let _chat = ctx.services().chats().find_by_id(params.chat_id).await?;
    let items = ctx
        .services()
        .artifacts()
        .find_by_chat_id(
            params.chat_id,
            nim::state::Page {
                cursor: query.cursor,
                limit: query.limit,
            },
        )
        .await?;

    Ok(axum::Json(items).into_response())
}

mod defaults {
    pub fn limit() -> usize {
        5
    }
}
