use nim::{signals::Signal, state::Message};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx, signal))]
pub async fn run(ctx: SignalContext, signal: Signal<Message>) -> Result<(), Error> {
    let message = signal.content;
    let content = message.content.to_string();
    let artifacts = nim::ai::pipelines::text::summarize::run(&[content.as_bytes()])?;

    if let Some(artifact) = artifacts.first() {
        let message = ctx
            .services()
            .messages()
            .update(
                message.id,
                nim::state::MessageUpdate {
                    summary: artifact.value.as_str().map(String::from),
                    confidence: artifact.confidence.map(|v| v as f32),
                    embedding: artifact.vector.clone(),
                    ..Default::default()
                },
            )
            .await?;

        ctx.signals().publish("nim.message.update", message).await?;
    }

    Ok(())
}
