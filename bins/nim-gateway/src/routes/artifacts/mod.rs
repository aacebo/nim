mod create;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/artifacts", axum::routing::post(create::route))
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
}
