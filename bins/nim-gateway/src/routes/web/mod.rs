mod connect;
mod get;
mod views;

use crate::Context;
use axum::http::{HeaderValue, header};
use tower_http::set_header::SetResponseHeaderLayer;

pub fn new() -> axum::Router<std::sync::Arc<Context>> {
    let templates = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let static_assets = |component: &str| tower_http::services::ServeDir::new(templates.join(component));

    axum::Router::new()
        .route("/", axum::routing::get(crate::diagnose(get::route)))
        .route("/chat", axum::routing::get(crate::diagnose(get::route)))
        .route("/chat/{chat_id}", axum::routing::get(crate::diagnose(get::route)))
        .route("/memory", axum::routing::get(crate::diagnose(get::route)))
        .route("/connect", axum::routing::any(crate::diagnose(connect::route)))
        .route("/views/chat", axum::routing::get(crate::diagnose(views::chats::route)))
        .route(
            "/view/chats/{chat_id}",
            axum::routing::get(crate::diagnose(views::chat::route)),
        )
        .route(
            "/views/messages/{message_id}",
            axum::routing::get(crate::diagnose(views::message::route)),
        )
        .route(
            "/views/artifacts/{artifact_id}",
            axum::routing::get(crate::diagnose(views::artifact::route)),
        )
        .route("/views/memory", axum::routing::get(crate::diagnose(views::memory::route)))
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
        .nest_service("/views/message/", static_assets("message"))
        .nest_service("/views/artifact/", static_assets("artifact"))
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
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
}
