use askama::Template;

#[derive(Clone, serde::Serialize)]
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

#[derive(Template)]
#[template(path = "message/message.html")]
pub struct MessageView {
    pub message: nim::state::Message,
    pub annotations: Vec<nim::state::MessageAnnotation>,
    pub artifacts: Vec<ArtifactView>,
}

#[derive(serde::Deserialize)]
pub struct Params {
    pub message_id: nim::state::MessageId,
}

pub async fn route(
    ctx: crate::http::HttpContext,
    params: axum::extract::Path<Params>,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let message = ctx.services().messages().find_by_id(params.message_id).await?;
    let view = MessageView {
        annotations: ctx.services().messages().find_annotations(message.id).await?,
        artifacts: ctx
            .services()
            .artifacts()
            .find_by_message_id(message.id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
        message,
    };

    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
