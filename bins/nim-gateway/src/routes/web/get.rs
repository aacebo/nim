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
        <link rel="stylesheet" href="/web/static/orb.css">
        <script src="/web/static/orb.js" defer></script>
    </head>
    <body>
        <main>
            <canvas id="orb" role="img" aria-label="A glowing blue and magenta orb with drifting beads"></canvas>
        </main>
    </body>
</html>"#
)]
pub struct Root;

pub async fn route() -> axum::response::Result<axum::response::Response> {
    let body = Root.render().map(Html).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    Ok((StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response())
}
