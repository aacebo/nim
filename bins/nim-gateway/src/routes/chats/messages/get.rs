use axum::response::IntoResponse;

use crate::{Error, http::HttpContext};

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
) -> axum::response::Result<axum::response::Response> {
    serde_valid::Validate::validate(&query.0).map_err(Error::from)?;
    let _chat = ctx.services().chats().find_by_id(params.chat_id).await.map_err(Error::from)?;
    let items = ctx
        .services()
        .messages()
        .find_by_chat_id(
            params.chat_id,
            nim::state::Page {
                cursor: query.cursor,
                limit: query.limit,
            },
        )
        .await
        .map_err(Error::from)?;

    let next = items
        .last()
        .filter(|_| items.len() == query.limit)
        .map(|message| nim::state::Cursor {
            id: message.id.into(),
            timestamp: message.created_at,
        });

    Ok(match next {
        Some(cursor) => ([("x-next-cursor", cursor.to_string())], axum::Json(items)).into_response(),
        None => axum::Json(items).into_response(),
    })
}

mod defaults {
    pub fn limit() -> usize {
        5
    }
}
