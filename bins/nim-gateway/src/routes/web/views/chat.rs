use askama::Template;

use crate::routes::web::views::chats::ChatsView;

#[derive(serde::Serialize)]
pub struct ChatView {
    pub chat: nim::state::Chat,
    pub messages: Vec<nim::state::Message>,
}

impl ChatView {
    pub async fn load(ctx: &crate::Context, chat_id: nim::state::ChatId) -> nim::error::Result<Self> {
        let chat = ctx.services().chats().find_by_id(chat_id).await?;

        let messages = ctx
            .services()
            .messages()
            .find_by_chat_id(
                chat.id,
                nim::state::Page {
                    cursor: None,
                    limit: 100,
                },
            )
            .await?
            .data;

        Ok(Self { chat, messages })
    }
}

#[derive(serde::Deserialize)]
pub struct Params {
    pub chat_id: nim::state::ChatId,
}

pub async fn route(
    ctx: crate::http::HttpContext,
    params: axum::extract::Path<Params>,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let view = ChatsView::load(&ctx, Some(params.chat_id)).await?;
    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
