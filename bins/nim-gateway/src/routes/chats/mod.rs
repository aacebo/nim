mod create;
mod messages;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .nest("/chats/{chat_id}", messages::new())
        .route("/chats/messages", axum::routing::post(create::route))
}
