use askama::Template;

#[derive(Template)]
#[template(path = "memory/memory.html")]
pub(super) struct MemoryView;
