mod artifacts;
mod chats;
mod health;
mod memories;
mod search;
mod web;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .merge(artifacts::new())
        .merge(memories::new())
        .merge(chats::new())
        .nest("/web", web::new())
        .route("/health", axum::routing::get(health::route))
        .route("/search", axum::routing::get(search::route))
}
