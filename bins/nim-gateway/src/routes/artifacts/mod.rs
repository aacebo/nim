mod content;
mod create;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/artifacts", axum::routing::post(crate::diagnose(create::route)))
        .route(
            "/artifacts/{artifact_id}/content",
            axum::routing::get(crate::diagnose(content::route)),
        )
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
}
