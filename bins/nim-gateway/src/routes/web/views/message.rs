use askama::Template;

#[derive(Template)]
#[template(path = "message/message.html")]
pub struct MessageView {
    pub message: nim::state::Message,
    pub annotations: Vec<nim::state::MessageAnnotation>,
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
        message,
    };

    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
