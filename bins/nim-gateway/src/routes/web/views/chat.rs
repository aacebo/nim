use askama::Template;

#[derive(Template)]
#[template(path = "chat/chat.html")]
pub struct ChatView {
    pub chat: Option<nim::state::Chat>,
    pub messages: Vec<nim::state::Message>,
}

impl ChatView {
    pub async fn load(ctx: &crate::Context) -> nim::error::Result<Self> {
        let Some(chat) = ctx.services().chats().find_latest().await? else {
            return Ok(Self {
                chat: None,
                messages: Vec::new(),
            });
        };

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

        Ok(Self {
            chat: Some(chat),
            messages,
        })
    }
}

pub async fn route(ctx: crate::http::HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let view = ChatView::load(&ctx).await?;
    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
