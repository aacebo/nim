use askama::Template;

#[derive(Template)]
#[template(path = "artifact/artifact.html")]
pub struct ArtifactView {
    pub artifact: nim::state::Artifact,
    pub annotations: Vec<nim::state::ArtifactAnnotation>,
}

#[derive(serde::Deserialize)]
pub struct Params {
    pub artifact_id: nim::state::ArtifactId,
}

pub async fn route(
    ctx: crate::http::HttpContext,
    params: axum::extract::Path<Params>,
) -> nim::error::Result<impl axum::response::IntoResponse> {
    use nim::error::IntoDiagnostic;

    let artifact = ctx.services().artifacts().find_by_id(params.artifact_id).await?;
    let view = ArtifactView {
        annotations: ctx.services().artifacts().find_annotations(artifact.id).await?,
        artifact,
    };

    let body = view.render().map_err(|error| error.into_diagnostic())?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], axum::response::Html(body)))
}
