use axum::response::IntoResponse;
use serde_valid::Validate;

use crate::http::HttpContext;

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
    query: axum::extract::Query<Query>,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    query.validate()?;
    let items = ctx
        .services()
        .chats()
        .find(nim::state::Page {
            cursor: query.cursor,
            limit: query.limit,
        })
        .await?;

    Ok(axum::Json(items).into_response())
}

mod defaults {
    pub fn limit() -> usize {
        5
    }
}
