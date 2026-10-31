use axum::response::IntoResponse;

use crate::http::HttpContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub content: Vec<nim::state::MessageContent>,
}

pub async fn route(ctx: HttpContext, body: axum::Json<Body>) -> axum::response::Result<axum::response::Response> {
    let chat = ctx
        .services()
        .chats()
        .create()
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    let message = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: chat.id,
            role: nim::state::Role::User,
            content: body.content.clone(),
        })
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    ctx.signals()
        .publish("nim.chat.create", chat)
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    ctx.signals()
        .publish("nim.message.create", message.clone())
        .await
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(message)).into_response())
}
