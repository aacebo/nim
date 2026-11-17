use askama::Template;

#[derive(Template)]
#[template(path = "memory/memory.html")]
pub struct MemoryView {
    pub records: Vec<nim::state::Memory>,
}

impl MemoryView {
    pub async fn load(ctx: &crate::Context) -> nim::error::Result<Self> {
        use nim::error::IntoDiagnostic;

        let mut records = Vec::new();
        let mut cursor = None;
        let mut cursors = Vec::new();

        loop {
            let page = ctx
                .services()
                .memories()
                .find(nim::state::Page { cursor, limit: 100 })
                .await?;
            records.extend(page.data);
            let Some(next) = page.next else { break };

            if cursors.contains(&next) {
                return Err(std::io::Error::other("memory cursor did not advance").into_diagnostic());
            }

            cursors.push(next);
            cursor = Some(next);
        }

        Ok(Self { records })
    }
}

pub async fn route(ctx: crate::http::HttpContext) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let view = MemoryView::load(&ctx).await?;
    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
