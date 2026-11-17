use askama::Template;

#[derive(Template)]
#[template(path = "orb/orb.html")]
pub struct OrbView;
