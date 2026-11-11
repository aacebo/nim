mod get;
mod recall;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new()
        .route("/memories", axum::routing::get(get::route))
        .route("/memories/recall", axum::routing::post(recall::route))
}
