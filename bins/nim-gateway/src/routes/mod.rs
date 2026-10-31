mod artifacts;
mod chats;
mod health;
mod memories;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .merge(artifacts::new())
        .merge(memories::new())
        .merge(chats::new())
        .route("/health", axum::routing::get(health::route))
}
