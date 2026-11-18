use nim::{runtime::Render, signals::Signal, state::Message};

use crate::SignalContext;

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> nim::error::Result<()> {
    let message = signal.content;
    let content = message.content.render(&ctx.services()).await?;
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .chat(&[&content], nim::runtime::ai::TextChatOptions::default())
        .await?;

    let response = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: message.chat_id,
            parent_id: message.parent_id,
            role: nim::state::Role::Agent,
            content: nim::state::MessageContent::Array(
                artifacts
                    .into_iter()
                    .map(|v| nim::state::MessageContent::String(v.text))
                    .collect(),
            ),
        })
        .await?;

    ctx.signals().publish("nim.message.response", response).await?;
    Ok(())
}
