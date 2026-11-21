use nim::{runtime::Transform, signals::Signal, state::Message};

use crate::SignalContext;

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> nim::error::Result<()> {
    let message = signal.content;
    let mut messages = ctx.services().messages().find_history(message.id, 20).await?.data;

    messages.sort_by_key(|a| a.created_at);
    messages.extend([message.clone()]);

    let assistant = ctx
        .runtime()
        .pipelines()
        .text()
        .chat(nim::runtime::ai::TextChatOptions {
            chat: nim::ai::ChatOptions {
                messages: messages.transform(&ctx.services()).await?,
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;

    let response = ctx
        .services()
        .messages()
        .create(nim::state::MessageCreate {
            chat_id: message.chat_id,
            parent_id: message.parent_id,
            role: nim::state::Role::Agent,
            content: nim::state::MessageContent::Single(nim::state::MessageContentPart::String(
                assistant
                    .content
                    .and_then(|content| match content {
                        nim::ai::ChatMessageContent::Text(text) => Some(text),
                        nim::ai::ChatMessageContent::Parts(_) => None,
                    })
                    .unwrap_or_default(),
            )),
        })
        .await?;

    ctx.signals().publish("nim.message.response", response).await?;
    Ok(())
}
