mod files;
mod health;
mod search;
mod text;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .merge(files::new())
        .merge(text::new())
        .route("/health", axum::routing::get(health::route))
        .route("/search", axum::routing::get(search::route))
}
