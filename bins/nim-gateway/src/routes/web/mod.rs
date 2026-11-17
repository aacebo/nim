mod connect;
mod get;
mod views;

use crate::Context;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    let templates = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let static_assets = |component: &str| tower_http::services::ServeDir::new(templates.join(component));

    axum::Router::new()
        .route("/", axum::routing::get(crate::diagnose(get::route)))
        .route("/connect", axum::routing::any(crate::diagnose(connect::route)))
        .route_service(
            "/static/index.css",
            tower_http::services::ServeFile::new(templates.join("index.css")),
        )
        .route_service(
            "/static/index.js",
            tower_http::services::ServeFile::new(templates.join("index.js")),
        )
        .nest_service("/static/orb", static_assets("orb"))
        .nest_service("/static/chat", static_assets("chat"))
        .nest_service("/static/memory", static_assets("memory"))
        .route(
            "/static/dev-tools/dev-tools.js",
            axum::routing::get(|| async {
                (
                    [
                        (axum::http::header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
                        (axum::http::header::CACHE_CONTROL, "no-store"),
                    ],
                    include_str!("../../../templates/dev-tools/dev-tools.js"),
                )
            }),
        )
        .route(
            "/static/dev-tools/dev-tools.css",
            axum::routing::get(|| async {
                (
                    [
                        (axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8"),
                        (axum::http::header::CACHE_CONTROL, "no-store"),
                    ],
                    include_str!("../../../templates/dev-tools/dev-tools.css"),
                )
            }),
        )
        .nest_service("/static/artifacts", static_assets("artifacts"))
}
