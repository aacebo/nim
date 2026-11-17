use askama::Template;

#[derive(Template)]
#[template(path = "dev-tools/dev-tools.html")]
pub(super) struct DevToolsView;
