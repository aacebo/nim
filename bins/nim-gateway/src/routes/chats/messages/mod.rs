mod create;
mod get;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/messages", axum::routing::post(crate::diagnose(create::route)))
        .route("/messages", axum::routing::get(crate::diagnose(get::route)))
}
