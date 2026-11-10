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
        <link rel="stylesheet" href="/web/static/hud.css?v=11">
        <link rel="stylesheet" href="/web/static/memory.css?v=8">
        <script src="/web/static/client.js" defer></script>
        <script src="/web/static/orb.js?v=5" defer></script>
        <script src="/web/static/memory.js?v=7" defer></script>
        <script src="/web/static/hud.js?v=10" defer></script>
    </head>
    <body>
        <main class="orb-scene" data-hud="idle">
            <svg class="hud-grid" viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
                <defs>
                    <symbol id="hud-cube" viewBox="0 0 32 32">
                        <path fill="var(--cube-top, #e2f0f7)" d="m16 2 13 7-13 7L3 9Z"/>
                        <path fill="var(--cube-left, #9eb6c8)" d="m3 9 13 7v14L3 23Z"/>
                        <path fill="var(--cube-right, #668295)" d="m16 16 13-7v14l-13 7Z"/>
                    </symbol>
                    <symbol id="hud-cube-front" viewBox="0 0 32 32">
                        <path fill="#e2f0f7" d="m9 2 20 5-8 6L2 8Z"/>
                        <path fill="#9eb6c8" d="m2 8 19 5v17L2 24Z"/>
                        <path fill="#668295" d="m21 13 8-6v17l-8 6Z"/>
                    </symbol>
                    <symbol id="hud-cube-side" viewBox="0 0 32 32">
                        <path fill="#e2f0f7" d="m23 3 7 9-20 5L2 9Z"/>
                        <path fill="#9eb6c8" d="m2 9 8 8v12l-8-8Z"/>
                        <path fill="#668295" d="m10 17 20-5v12l-20 5Z"/>
                    </symbol>
                    <symbol id="hud-cube-under" viewBox="0 0 32 32">
                        <path fill="#d4e5ef" d="M3 8 17 2v15L3 23Z"/>
                        <path fill="#9eb6c8" d="m17 2 12 8v15l-12-8Z"/>
                        <path fill="#668295" d="m3 23 14-6 12 8-14 6Z"/>
                    </symbol>
                </defs>
                <path class="grid-lines" d="M600 350-450 800M600 350-150 800M600 350 150 800M600 350 450 800M600 350 750 800M600 350 1050 800M600 350 1350 800M600 350 1650 800M0 388h1200M0 418h1200M0 466h1200M0 542h1200M0 652h1200M0 798h1200M0 350h1200M600 350-150 0M600 350 175 0M600 350 460 0M600 350 740 0M600 350 1025 0M600 350 1350 0"/>
            </svg>
            <div class="hud-cubes" aria-hidden="true">
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 15%; --y: 17%; --size: 24px; --tone: .28; --angle: -24deg; --drift-x: 9px; --drift-y: -7px; --drift-time: 18s; --turn-time: 34s; --phase: -3s">
                    <g><use href="#hud-cube-front"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 82%; --y: 18%; --size: 58px; --tone: .18; --angle: 32deg; --drift-x: -12px; --drift-y: 6px; --drift-time: 21s; --turn-time: 47s; --phase: -8s; --turn: -360deg">
                    <g><use href="#hud-cube-side"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 9%; --y: 37%; --size: 20px; --tone: .32; --angle: 12deg; --drift-x: 8px; --drift-y: 9px; --drift-time: 16s; --turn-time: 39s; --phase: -1s">
                    <g><use href="#hud-cube-under"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 89%; --y: 39%; --size: 46px; --tone: .2; --angle: -39deg; --drift-x: -14px; --drift-y: -8px; --drift-time: 22s; --turn-time: 50s; --phase: -10s; --turn: -360deg">
                    <g><use href="#hud-cube"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 34%; --y: 12%; --size: 27px; --tone: .23; --angle: -8deg; --drift-x: 11px; --drift-y: 6px; --drift-time: 14s; --turn-time: 32s; --phase: -4s">
                    <g><use href="#hud-cube-side"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 13%; --y: 69%; --size: 62px; --tone: .19; --angle: 38deg; --drift-x: 14px; --drift-y: -6px; --drift-time: 20s; --turn-time: 52s; --phase: -6s; --turn: -360deg">
                    <g><use href="#hud-cube-under"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 70%; --y: 9%; --size: 22px; --tone: .3; --angle: 66deg; --drift-x: -9px; --drift-y: 11px; --drift-time: 17s; --turn-time: 37s; --phase: -12s">
                    <g><use href="#hud-cube-front"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 88%; --y: 77%; --size: 54px; --tone: .2; --angle: 19deg; --drift-x: -12px; --drift-y: 8px; --drift-time: 23s; --turn-time: 45s; --phase: -5s; --turn: -360deg">
                    <g><use href="#hud-cube-side"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 23%; --y: 88%; --size: 29px; --tone: .27; --angle: -47deg; --drift-x: 13px; --drift-y: -9px; --drift-time: 19s; --turn-time: 43s; --phase: -11s">
                    <g><use href="#hud-cube"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 67%; --y: 83%; --size: 44px; --tone: .22; --angle: 54deg; --drift-x: 10px; --drift-y: 7px; --drift-time: 15s; --turn-time: 48s; --phase: -7s; --turn: -360deg">
                    <g><use href="#hud-cube-front"/></g>
                </svg>
                <svg class="background-cube" viewBox="0 0 32 32" style="--x: 36%; --y: 68%; --size: 19px; --tone: .23; --angle: -16deg; --drift-x: -11px; --drift-y: -10px; --drift-time: 24s; --turn-time: 36s; --phase: -9s">
                    <g><use href="#hud-cube-under"/></g>
                </svg>
            </div>
            <button id="orb-trigger" type="button" aria-label="Orb actions" aria-controls="orb-hub" aria-expanded="false">
                <canvas id="orb" aria-hidden="true"></canvas>
            </button>
            <nav id="orb-hub" aria-label="Orb actions" inert>
                <button id="chat-action" class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#hud-cube"/></svg>
                    <span class="hub-label">Chat</span>
                </button>
                <button id="memory-action" class="hub-action" type="button" aria-controls="memory-view">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#hud-cube"/></svg>
                    <span class="hub-label">Memory</span>
                </button>
                <button class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#hud-cube"/></svg>
                    <span class="hub-label">Settings</span>
                </button>
                <button class="hub-action" type="button" aria-controls="action-panel">
                    <svg class="hub-symbol" viewBox="0 0 32 32" aria-hidden="true"><use href="#hud-cube"/></svg>
                    <span class="hub-label">Action 4</span>
                </button>
                <button id="hub-back" type="button">Back</button>
            </nav>
            <div class="action-view">
                <section id="action-panel" aria-labelledby="action-title" inert>
                    <header class="action-header">
                        <div class="action-heading">
                            <svg viewBox="0 0 32 32" aria-hidden="true"><use href="#hud-cube"/></svg>
                            <h2 id="action-title"></h2>
                        </div>
                        <button id="action-back" type="button" aria-label="Back">
                            <span>Back</span>
                            <svg viewBox="0 0 16 16" aria-hidden="true"><path d="m10 3-5 5 5 5"/></svg>
                        </button>
                    </header>
                    <form id="chat-form" hidden>
                        <div class="chat-fields">
                            <span class="chat-input-field">
                                <input id="chat-input" type="text" aria-label="Message" placeholder="Write a message…" autocomplete="off" required>
                            </span>
                            <button id="chat-send" type="submit">Send</button>
                        </div>
                        <p id="chat-error" role="alert"></p>
                    </form>
                    <p id="chat-status" role="status"></p>
                </section>
                <aside id="chat-transcript" aria-labelledby="chat-transcript-title" hidden>
                    <h2 id="chat-transcript-title">Transcript</h2>
                    <p id="chat-history-status" role="status"></p>
                    <div class="chat-scroll" role="region" aria-label="Messages" tabindex="0">
                        <ol id="chat-messages"></ol>
                    </div>
                </aside>
            </div>
            <section id="memory-view" role="dialog" aria-modal="true" aria-labelledby="memory-title" hidden>
                <div class="memory-atmosphere" aria-hidden="true">
                    <svg viewBox="0 0 1200 800" preserveAspectRatio="xMidYMid slice">
                        <defs>
                            <linearGradient id="memory-floor-light" x2="0" y2="1">
                                <stop stop-color="#506c90" stop-opacity=".04"/>
                                <stop offset="1" stop-color="#506c90" stop-opacity=".23"/>
                            </linearGradient>
                            <linearGradient id="memory-horizon-light" gradientUnits="userSpaceOnUse" x1="0" x2="1200" y1="440" y2="440">
                                <stop stop-color="#ffdba2" stop-opacity=".9"/>
                                <stop offset=".31" stop-color="#e6b271" stop-opacity=".04"/>
                                <stop offset=".62" stop-color="#e6b271" stop-opacity=".12"/>
                                <stop offset=".83" stop-color="#f2c589" stop-opacity=".7"/>
                                <stop offset="1" stop-color="#ffdba2" stop-opacity=".08"/>
                            </linearGradient>
                        </defs>
                        <path fill="url(#memory-floor-light)" d="M0 440h1200v360H0Z"/>
                        <path class="memory-horizon-glow" stroke="url(#memory-horizon-light)" d="M0 440h1200"/>
                        <path class="memory-horizon" stroke="url(#memory-horizon-light)" d="M0 440h1200"/>
                        <g id="memory-horizon-points"/>
                    </svg>
                </div>
                <header class="memory-header">
                    <div>
                        <p class="memory-kicker">Nim archive</p>
                        <h2 id="memory-title">Memory</h2>
                    </div>
                    <div class="memory-controls">
                        <div class="memory-modes" role="group" aria-label="Memory view">
                            <button id="memory-timeline-toggle" type="button" aria-pressed="true" aria-controls="memory-timeline-stage">Timeline</button>
                            <button id="memory-graph-toggle" type="button" aria-pressed="false" aria-controls="memory-graph">Graph</button>
                        </div>
                        <button id="memory-back" type="button"><span aria-hidden="true">‹</span> Back</button>
                    </div>
                </header>
                <div id="memory-timeline-stage">
                    <canvas id="memory-timeline-strand" aria-hidden="true"></canvas>
                    <div id="memory-timeline" role="region" aria-label="Memory timeline">
                        <ol id="memory-strand"></ol>
                    </div>
                    <div class="memory-selection" aria-hidden="true">
                        <div class="memory-selection-index"><span id="memory-selection-index"></span><span>Record</span></div>
                        <div class="memory-selection-copy">
                            <span class="memory-selection-kicker">Memory sequence</span>
                            <strong id="memory-selection-title"></strong>
                            <span id="memory-selection-type"></span>
                            <span id="memory-selection-date"></span>
                        </div>
                    </div>
                </div>
                <div id="memory-graph" role="region" aria-label="Memory relations graph" tabindex="0" hidden>
                    <div class="memory-graph-surface">
                        <svg id="memory-edges" viewBox="0 0 1040 620" aria-hidden="true"></svg>
                        <div id="memory-nodes"></div>
                    </div>
                </div>
                <footer class="memory-footer">
                    <span>Demo memories</span>
                    <span id="memory-view-status" role="status"></span>
                </footer>
                <section id="memory-details" role="dialog" aria-labelledby="memory-detail-title" hidden>
                    <div class="memory-detail-header">
                        <p class="memory-kicker">Memory record</p>
                        <button id="memory-detail-close" type="button" aria-label="Close memory details">×</button>
                    </div>
                    <h3 id="memory-detail-title"></h3>
                    <div id="memory-detail-art" aria-hidden="true" hidden></div>
                    <p id="memory-detail-meta"></p>
                    <p id="memory-detail-summary"></p>
                    <button id="memory-expand" type="button" hidden>Expand relations</button>
                </section>
            </section>
        </main>
    </body>
</html>"##
)]
pub struct Root;

pub async fn route() -> axum::response::Result<axum::response::Response> {
    let body = Root.render().map(Html).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    Ok((StatusCode::OK, [(header::CACHE_CONTROL, "no-store")], Html(body)).into_response())
}
