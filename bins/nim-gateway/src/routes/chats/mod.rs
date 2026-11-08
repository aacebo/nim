mod artifacts;
mod create;
mod get;
mod messages;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .nest("/chats/{chat_id}", messages::new().merge(artifacts::new()))
        .route("/chats/messages", axum::routing::post(create::route))
        .route("/chats", axum::routing::get(get::route))
}
