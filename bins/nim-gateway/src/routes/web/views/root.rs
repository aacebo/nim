use askama::Template;

use super::orb::OrbView;

#[derive(Template)]
#[template(path = "index.html")]
pub struct RootView {
    pub orb: OrbView,
    pub has_memories: bool,
}

impl RootView {
    pub async fn load(ctx: &crate::Context) -> nim::error::Result<Self> {
        let page = ctx
            .services()
            .memories()
            .find(nim::state::Page { cursor: None, limit: 5 })
            .await?;
        Ok(Self {
            orb: OrbView,
            has_memories: !page.data.is_empty(),
        })
    }
}
