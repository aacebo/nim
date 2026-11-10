use askama::Template;
use axum::{
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};

#[derive(askama::Template)]
#[template(
    ext = "html",
    source = r##"
<!doctype html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
        <title>Nim</title>
        <link rel="stylesheet" href="/web/static/orb.css?v=2">
        <link rel="stylesheet" href="/web/static/hud.css?v=5">
        <script src="/web/static/client.js" defer></script>
        <script src="/web/static/orb.js?v=5" defer></script>
        <script src="/web/static/hud.js?v=4" defer></script>
    </head>
    <body>
        <main class="orb-scene" data-hud="idle">
            <svg class="animus-grid" viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
                <defs>
                    <symbol id="animus-cube" viewBox="0 0 32 32">
                        <path fill="var(--cube-top, #e2f0f7)" d="m16 2 13 7-13 7L3 9Z"/>
                        <path fill="var(--cube-left, #9eb6c8)" d="m3 9 13 7v14L3 23Z"/>
                        <path fill="var(--cube-right, #668295)" d="m16 16 13-7v14l-13 7Z"/>
                    </symbol>
                </defs>
                <path class="grid-lines" d="M600 350-450 800M600 350-150 800M600 350 150 800M600 350 450 800M600 350 750 800M600 350 1050 800M600 350 1350 800M600 350 1650 800M0 388h1200M0 418h1200M0 466h1200M0 542h1200M0 652h1200M0 798h1200M0 350h1200M600 350-150 0M600 350 175 0M600 350 460 0M600 350 740 0M600 350 1025 0M600 350 1350 0"/>
                <use href="#animus-cube" x="127" y="145" width="54" height="54" opacity=".14"/>
                <use href="#animus-cube" x="237" y="495" width="29" height="29" opacity=".18"/>
                <use href="#animus-cube" x="462" y="130" width="27" height="27" opacity=".12"/>
                <use href="#animus-cube" x="726" y="651" width="32" height="32" opacity=".14"/>
                <use href="#animus-cube" x="918" y="105" width="24" height="24" opacity=".22"/>
                <use href="#animus-cube" x="988" y="512" width="77" height="77" opacity=".13"/>
                <use href="#animus-cube" x="832" y="673" width="35" height="35" opacity=".17"/>
            </svg>
            <button id="orb-trigger" type="button" aria-label="Orb actions" aria-controls="animus-hub" aria-expanded="false">
                <canvas id="orb" aria-hidden="true"></canvas>
            </button>
            <nav id="animus-hub" aria-label="Animus hub" inert>
                <button id="chat-action" class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#animus-cube"/></svg>
                    <span class="hub-label">Chat</span>
                </button>
                <button class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#animus-cube"/></svg>
                    <span class="hub-label">Action 2</span>
                </button>
                <button class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#animus-cube"/></svg>
                    <span class="hub-label">Action 3</span>
                </button>
                <button class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#animus-cube"/></svg>
                    <span class="hub-label">Action 4</span>
                </button>
                <button id="hub-back" type="button">Back</button>
            </nav>
            <section id="action-panel" aria-labelledby="action-title" inert>
                <header class="action-header">
                    <div class="action-heading">
                        <svg viewBox="0 0 32 32" aria-hidden="true"><use href="#animus-cube"/></svg>
                        <h2 id="action-title"></h2>
                    </div>
                    <button id="action-back" type="button" aria-label="Back">
                        <span>Back</span>
                        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="m10 3-5 5 5 5"/></svg>
                    </button>
                </header>
                <form id="chat-form" hidden>
                    <div class="chat-fields">
                        <input id="chat-input" type="text" aria-label="Message" placeholder="Write a message…" autocomplete="off" required>
                        <button id="chat-send" type="submit">Send</button>
                    </div>
                    <p id="chat-error" role="alert"></p>
                </form>
            </section>
            <p id="chat-status" role="status"></p>
        </main>
    </body>
</html>"##
)]
pub struct Root;

pub async fn route() -> axum::response::Result<axum::response::Response> {
    let body = Root.render().map(Html).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    Ok((StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response())
}
