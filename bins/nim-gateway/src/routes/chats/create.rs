use axum::response::IntoResponse;

use crate::{extract, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Body {
    #[validate(min_length = 1)]
    pub content: String,

    #[serde(default)]
    pub artifacts: Vec<nim::state::ArtifactId>,
}

pub async fn route(ctx: HttpContext, body: extract::Json<Body>) -> nim::error::Result<impl axum::response::IntoResponse> {
    let chat = ctx.services().chats().create().await?;
    let message = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: chat.id,
            reply_to_id: None,
            role: nim::state::Role::User,
            content: body.content.parse()?,
            artifacts: body.artifacts.clone(),
        })
        .await?;

    ctx.signals().publish("nim.chat.create", chat).await?;
    ctx.signals().publish("nim.message.create", message.clone()).await?;
    Ok((axum::http::StatusCode::CREATED, axum::Json(message)).into_response())
}
