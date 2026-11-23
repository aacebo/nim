use askama::Template;

#[derive(Template)]
#[template(path = "memory/memory.html")]
pub struct MemoryView {
    pub records: Vec<nim::state::Memory>,
}

impl MemoryView {
    pub async fn load(ctx: &crate::Context) -> nim::error::Result<Self> {
        let records = ctx
            .services()
            .memories()
            .find(nim::state::Page {
                cursor: None,
                limit: 100,
            })
            .await?
            .data;

        Ok(Self { records })
    }
}

pub async fn route(ctx: crate::http::HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let view = MemoryView::load(&ctx).await?;
    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
