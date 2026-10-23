mod upload;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/files", axum::routing::post(upload::route))
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
}
