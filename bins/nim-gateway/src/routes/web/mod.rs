mod get;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");

    axum::Router::new().route("/", axum::routing::get(get::route)).nest_service(
        "/static",
        tower_http::services::ServeDir::new(static_dir).append_index_html_on_directories(true),
    )
}
