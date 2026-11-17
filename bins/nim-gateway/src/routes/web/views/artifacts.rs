use askama::Template;

#[derive(Template)]
#[template(path = "artifacts/artifacts.html")]
pub struct ArtifactsView;

pub async fn route() -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let body = ArtifactsView.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
