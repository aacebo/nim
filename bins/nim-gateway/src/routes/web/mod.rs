mod get;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    axum::Router::new().route("/web", axum::routing::get(get::route))
}
