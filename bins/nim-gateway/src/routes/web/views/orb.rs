use askama::Template;

#[derive(Template)]
#[template(path = "orb/orb.html")]
pub(super) struct OrbView;
