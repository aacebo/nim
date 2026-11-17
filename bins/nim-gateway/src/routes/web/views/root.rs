use askama::Template;

use super::{artifacts::ArtifactsView, chat::ChatView, dev_tools::DevToolsView, memory::MemoryView, orb::OrbView};

#[derive(Template)]
#[template(path = "index.html")]
pub(super) struct RootView {
    pub(super) orb: OrbView,
    pub(super) chat: ChatView,
    pub(super) memory: MemoryView,
    pub(super) dev_tools: DevToolsView,
    pub(super) artifacts: ArtifactsView,
}
