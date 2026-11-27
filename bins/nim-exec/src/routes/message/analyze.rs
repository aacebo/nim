use nim::{signals::Signal, state::Message};

use crate::SignalContext;

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> nim::error::Result<()> {
    let message = signal.content;
    let mut messages = ctx
        .services()
        .messages()
        .find_history(message.chat_id, message.id, 20)
        .await?
        .data;

    messages.sort_by_key(|value| value.created_at);
    messages.push(message.clone());
    let mut history = Vec::with_capacity(messages.len());

    for item in &messages {
        history.push(nim::runtime::render_message(&ctx.services(), item).await?);
    }

    let assistant = ctx
        .runtime()
        .pipelines()
        .text()
        .complete(nim::runtime::ai::TextCompleteOptions {
            chat: nim::ai::Chat {
                messages: history,
                parse: nim::chat::ParseMode::Lenient,
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;

    let content = assistant
        .content
        .ok_or_else(|| nim::error::Diagnostic::message("assistant response has no content"))?;
    let response = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: message.chat_id,
            reply_to_id: message.reply_to_id,
            role: nim::state::Role::Agent,
            content,
            artifacts: Default::default(),
        })
        .await?;

    ctx.signals().publish("nim.message.response", response).await?;
    Ok(())
}
