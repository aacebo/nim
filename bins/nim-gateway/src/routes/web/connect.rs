use axum::extract::ws::{Message, WebSocketUpgrade};
use futures_util::SinkExt;

use crate::http::HttpContext;

pub async fn route(ctx: HttpContext, ws: WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(async move |mut socket| {
        let mut subscription = ctx.sockets().open().await;

        loop {
            tokio::select! {
                message = socket.recv() => match message {
                    Some(Ok(Message::Close(_))) => {
                        let _ = socket.flush().await;
                        break;
                    }
                    Some(Err(_)) | None => break,
                    Some(Ok(_)) => {}
                },
                signal = subscription.recv() => {
                    let Some(signal) = signal else {
                        break;
                    };

                    let Ok(message) = serde_json::to_vec(&signal) else {
                        break;
                    };

                    if socket.send(Message::Binary(message.into())).await.is_err() {
                        break;
                    }
                }
            }
        }

        ctx.sockets().close(subscription).await;
    })
}
