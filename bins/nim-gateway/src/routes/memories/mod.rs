mod get;
mod recall;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/memories", axum::routing::get(crate::diagnose(get::route)))
        .route("/memories/recall", axum::routing::post(crate::diagnose(recall::route)))
}
