use askama::Template;

use super::chat::ChatView;

#[derive(Template)]
#[template(path = "chat/chat.html")]
pub struct ChatsView {
    pub chat: Option<ChatView>,
}

impl ChatsView {
    pub async fn load(ctx: &crate::Context, chat_id: Option<nim::state::ChatId>) -> nim::error::Result<Self> {
        let chat = match chat_id {
            Some(chat_id) => Some(ChatView::load(ctx, chat_id).await?),
            None => None,
        };

        Ok(Self { chat })
    }
}

pub async fn route(ctx: crate::http::HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let view = ChatsView::load(&ctx, None).await?;
    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
