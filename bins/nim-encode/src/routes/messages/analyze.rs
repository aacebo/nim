use nim::{
    signals::Signal,
    state::{Message, Page},
};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> Result<(), Error> {
    let chat = ctx.services().chats().find_by_id(signal.content.chat_id).await?;
    let cursor = chat
        .last_analyzed_id
        .zip(chat.last_analyzed_at)
        .map(|(id, at)| (id.into(), at));
    let messages = ctx
        .services()
        .messages()
        .find_by_chat_id(chat.id, Page { cursor, limit: 20 })
        .await?;
    let mut summaries = Vec::new();

    if let Some(summary) = &chat.summary {
        summaries.push(summary.clone());
    }

    let mut last_analyzed = None;

    for message in &messages {
        let Some(summary) = message.summary.as_deref().filter(|summary| !summary.trim().is_empty()) else {
            break;
        };

        summaries.push(summary.to_string());
        last_analyzed = Some((message.id, message.created_at));
    }

    let Some((last_analyzed_id, last_analyzed_at)) = last_analyzed else {
        return Ok(());
    };

    let text = summaries.join("\n");
    let artifacts = ctx
        .runtime()
        .pipelines()
        .text()
        .summarize(
            &[&text],
            nim::runtime::ai::TextSummarizeOptions {
                embedding: nim::ai::pipelines::text::embeddings::Options {
                    model: nim::ai::models::refer("unsloth/embeddinggemma-300m")?,
                    embedding: nim::ai::models::EmbedOptions {
                        mode: nim::ai::models::EmbedMode::Document,
                        dimensions: Some(768),
                        ..Default::default()
                    },
                },
                task: nim::runtime::TaskOptions {
                    sources: vec![chat.id.into(), signal.content.id.into()],
                    ..Default::default()
                },
                model: nim::ai::models::refer("Qwen/Qwen2.5-3B-Instruct-GGUF")?,
                ..Default::default()
            },
        )
        .await?;

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
                last_analyzed_id: Some(last_analyzed_id),
                last_analyzed_at: Some(last_analyzed_at),
            },
        )
        .await?;

    ctx.signals().publish("nim.chat.update", chat).await?;
    Ok(())
}
