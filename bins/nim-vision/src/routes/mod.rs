mod upload;

use crate::State;

pub fn new() -> axum::Router<std::sync::Arc<State>> {
    axum::Router::new().route("/", axum::routing::post(upload::route))
}
