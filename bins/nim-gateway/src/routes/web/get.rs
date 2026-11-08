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
        <title>Hello World</title>
    </head>
    <body>
        <h1>Hello World</h1>
    </body>
</html>"#
)]
pub struct Root;

pub async fn route() -> axum::response::Result<axum::response::Response> {
    let body = Root.render().map(Html).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    Ok((StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response())
}
