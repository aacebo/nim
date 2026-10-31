mod create;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new().route("/chats/messages", axum::routing::post(create::route))
}
