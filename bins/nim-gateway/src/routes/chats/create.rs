use axum::response::IntoResponse;

use crate::{Error, extract, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Body {
    #[validate]
    pub role: nim::state::Role,

    #[validate]
    pub content: nim::state::MessageContent,
}

pub async fn route(ctx: HttpContext, body: extract::Json<Body>) -> axum::response::Result<axum::response::Response> {
    let chat = ctx.services().chats().create().await.map_err(Error::from)?;
    let message = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: chat.id,
            parent_id: None,
            role: body.role,
            content: body.content.clone(),
        })
        .await
        .map_err(Error::from)?;

    ctx.signals().publish("nim.chat.create", chat).await.map_err(Error::from)?;
    ctx.signals()
        .publish("nim.message.create", message.clone())
        .await
        .map_err(Error::from)?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(message)).into_response())
}
