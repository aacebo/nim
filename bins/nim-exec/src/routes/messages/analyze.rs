use nim::{signals::Signal, state::Message};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> Result<(), Error> {
    let chat = ctx.services().chats().find_by_id(signal.content.chat_id).await?;
    let messages = ctx.services().messages().find_by_chat_id(chat.id, 20).await?;
    let mut summaries = Vec::new();

    if let Some(summary) = &chat.summary {
        summaries.push(summary.clone());
    }

    summaries.extend(messages.iter().filter_map(|m| m.summary.clone()));
    let text = summaries.join("\n");
    let artifacts = nim::ai::pipelines::text::summarize::run(&[text.as_bytes()])?;
    let Some(artifact) = artifacts.first() else {
        return Err(Error::custom(format!(
            "summary could be generated from artifact `{}`",
            signal.content.id
        )));
    };

    let chat = ctx
        .services()
        .chats()
        .update(
            chat.id,
            nim::state::ChatUpdate {
                summary: artifact.value.as_str().map(String::from),
                embedding: artifact.vector.clone(),
                confidence: artifact.confidence.map(|v| v as f32),
                last_analyzed_id: Some(signal.content.id),
            },
        )
        .await?;

    ctx.signals().publish("nim.chat.update", chat).await?;
    Ok(())
}
