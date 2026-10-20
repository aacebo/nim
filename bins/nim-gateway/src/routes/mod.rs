mod upload;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new().route("/", axum::routing::post(upload::route))
}
