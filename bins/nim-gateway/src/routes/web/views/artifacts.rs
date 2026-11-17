use askama::Template;

#[derive(Template)]
#[template(path = "artifacts/artifacts.html")]
pub(super) struct ArtifactsView;
