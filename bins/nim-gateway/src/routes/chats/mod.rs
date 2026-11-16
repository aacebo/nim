mod artifacts;
mod create;
mod get;
mod messages;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .nest("/chats/{chat_id}", messages::new().merge(artifacts::new()))
        .route("/chats/messages", axum::routing::post(crate::diagnose(create::route)))
        .route("/chats", axum::routing::get(crate::diagnose(get::route)))
}
