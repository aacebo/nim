use askama::Template;

#[derive(Template)]
#[template(path = "chat/chat.html")]
pub struct ChatView {
    pub chat: Option<nim::state::Chat>,
    pub messages: Vec<nim::state::Message>,
    pub artifacts: Vec<ArtifactView>,
}

#[derive(serde::Serialize)]
pub struct ArtifactView {
    pub id: nim::state::ArtifactId,
    pub version: nim::state::Version,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub media_type: Option<String>,
    pub size: Option<u64>,
}

impl From<nim::state::Artifact> for ArtifactView {
    fn from(value: nim::state::Artifact) -> Self {
        let (kind, media_type, size) = match value.content {
            nim::state::ArtifactContent::Text { .. } => ("text", Some("text/plain".into()), None),
            nim::state::ArtifactContent::File { media_type, size, .. } => ("file", media_type, Some(size)),
            nim::state::ArtifactContent::Resource { .. } => ("resource", None, None),
        };
        Self {
            id: value.id,
            version: value.version,
            name: value.name,
            kind,
            media_type,
            size,
        }
    }
}

impl ChatView {
    pub async fn load(ctx: &crate::Context) -> nim::error::Result<Self> {
        use nim::error::IntoDiagnostic;

        let Some(chat) = ctx.services().chats().find_latest().await? else {
            return Ok(Self {
                chat: None,
                messages: Vec::new(),
                artifacts: Vec::new(),
            });
        };

        let mut messages = Vec::new();
        let mut artifacts = Vec::new();
        let mut cursor = None;
        let mut cursors = Vec::new();

        loop {
            let page = ctx
                .services()
                .messages()
                .find_by_chat_id(chat.id, nim::state::Page { cursor, limit: 100 })
                .await?;
            messages.extend(page.data);
            let Some(next) = page.next else { break };

            if cursors.contains(&next) {
                return Err(std::io::Error::other("message cursor did not advance").into_diagnostic());
            }

            cursors.push(next);
            cursor = Some(next);
        }

        let mut cursor = None;
        let mut cursors = Vec::new();

        loop {
            let page = ctx
                .services()
                .artifacts()
                .find_by_chat_id(chat.id, nim::state::Page { cursor, limit: 100 })
                .await?;
            artifacts.extend(page.data);
            let Some(next) = page.next else { break };

            if cursors.contains(&next) {
                return Err(std::io::Error::other("artifact cursor did not advance").into_diagnostic());
            }

            cursors.push(next);
            cursor = Some(next);
        }

        Ok(Self {
            chat: Some(chat),
            messages,
            artifacts: artifacts.into_iter().map(Into::into).collect(),
        })
    }
}

pub async fn route(ctx: crate::http::HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let view = ChatView::load(&ctx).await?;
    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
