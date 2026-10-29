mod recall;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new().route("/memories/recall", axum::routing::post(recall::route))
}
