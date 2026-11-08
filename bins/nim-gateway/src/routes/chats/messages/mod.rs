mod create;
mod get;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/messages", axum::routing::post(create::route))
        .route("/messages", axum::routing::get(get::route))
}
