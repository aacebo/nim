use axum::response::IntoResponse;

use crate::extract;
use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Params {
    pub chat_id: nim::state::ChatId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Body {
    #[validate]
    pub content: nim::state::MessageContent,
}

pub async fn route(
    ctx: HttpContext,
    params: axum::extract::Path<Params>,
    body: extract::Json<Body>,
) -> axum::response::Result<axum::response::Response> {
    let chat = ctx
        .services()
        .chats()
        .find_by_id(params.chat_id)
        .await
        .map_err(|err| (axum::http::StatusCode::NOT_FOUND, err.to_string()).into_response())?;

    let message = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: chat.id,
            parent_id: None,
            role: nim::state::Role::User,
            content: body.content.clone(),
        })
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    ctx.signals()
        .publish("nim.message.create", message.clone())
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(message)).into_response())
}
