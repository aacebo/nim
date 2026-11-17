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
        .route("/views/chat", axum::routing::get(crate::diagnose(views::chat::route)))
        .route("/views/memory", axum::routing::get(crate::diagnose(views::memory::route)))
        .route(
            "/views/artifacts",
            axum::routing::get(crate::diagnose(views::artifacts::route)),
        )
        .route(
            "/views/dev-tools",
            axum::routing::get(crate::diagnose(views::dev_tools::route)),
        )
        .route_service(
            "/views/index.css",
            tower_http::services::ServeFile::new(templates.join("index.css")),
        )
        .route_service(
            "/views/index.js",
            tower_http::services::ServeFile::new(templates.join("index.js")),
        )
        .nest_service("/views/orb/", static_assets("orb"))
        .nest_service("/views/chat/", static_assets("chat"))
        .nest_service("/views/memory/", static_assets("memory"))
        .route(
            "/views/dev-tools/dev-tools.js",
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
            "/views/dev-tools/dev-tools.css",
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
        .nest_service("/views/artifacts/", static_assets("artifacts"))
}
