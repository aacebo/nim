mod files;
mod text;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new().nest("/files", files::new()).nest("/text", text::new())
}
