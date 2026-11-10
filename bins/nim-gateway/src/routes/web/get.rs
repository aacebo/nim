use askama::Template;
use axum::{
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};

#[derive(askama::Template)]
#[template(
    ext = "html",
    source = r#"
<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>Nim</title>
        <link rel="stylesheet" href="/web/static/orb.css?v=2">
        <script src="/web/static/client.js" defer></script>
        <script src="/web/static/orb.js?v=5" defer></script>
    </head>
    <body>
        <main>
            <button id="orb-trigger" type="button" aria-label="Ripple the glowing AI orb">
                <canvas id="orb" aria-hidden="true"></canvas>
            </button>
        </main>
    </body>
</html>"#
)]
pub struct Root;

pub async fn route() -> axum::response::Result<axum::response::Response> {
    let body = Root.render().map(Html).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    Ok((StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response())
}
