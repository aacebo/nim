mod artifacts;
mod chat;
mod dev_tools;
mod memory;
mod orb;
mod root;

pub(super) fn render() -> askama::Result<String> {
    use askama::Template;

    root::RootView {
        orb: orb::OrbView,
        chat: chat::ChatView,
        memory: memory::MemoryView,
        dev_tools: dev_tools::DevToolsView,
        artifacts: artifacts::ArtifactsView,
    }
    .render()
}
