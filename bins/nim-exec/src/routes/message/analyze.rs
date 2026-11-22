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
        .complete(nim::runtime::ai::TextCompleteOptions {
            chat: nim::ai::Chat {
                messages: messages.transform(&ctx.services()).await?,
                compat: nim::ai::CompatMode::Ignore,
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
            content: persist_content(&ctx, assistant.content).await?,
        })
        .await?;

    ctx.signals().publish("nim.message.response", response).await?;
    Ok(())
}

async fn persist_content(
    ctx: &SignalContext,
    content: Option<nim::ai::ChatMessageContent>,
) -> nim::error::Result<nim::state::MessageContent> {
    let Some(content) = content else {
        return Err(nim::error::Diagnostic::message("assistant response has no content"));
    };

    match content {
        nim::ai::ChatMessageContent::Text(text) => Ok(nim::state::MessageContent::String(text)),
        nim::ai::ChatMessageContent::Parts(parts) => {
            let mut output = Vec::with_capacity(parts.len());

            for part in parts {
                match part {
                    nim::ai::ChatMessageContentPart::Text { text } => {
                        output.push(nim::state::MessageContentPart::Text { text });
                    }
                    nim::ai::ChatMessageContentPart::Image(media) => {
                        output.push(nim::state::MessageContentPart::Artifact {
                            artifact_id: persist_media(ctx, media, "image/png").await?,
                        });
                    }
                    nim::ai::ChatMessageContentPart::Audio(media) => {
                        output.push(nim::state::MessageContentPart::Artifact {
                            artifact_id: persist_media(ctx, media, "audio/mpeg").await?,
                        });
                    }
                    nim::ai::ChatMessageContentPart::Video(media) => {
                        output.push(nim::state::MessageContentPart::Artifact {
                            artifact_id: persist_media(ctx, media, "video/mp4").await?,
                        });
                    }
                }
            }

            if output.is_empty() {
                return Err(nim::error::Diagnostic::message("assistant response has no content parts"));
            }

            Ok(nim::state::MessageContent::Parts(output))
        }
    }
}

async fn persist_media(
    ctx: &SignalContext,
    media: nim::ai::Media,
    media_type: &str,
) -> nim::error::Result<nim::state::ArtifactId> {
    let (name, content, summary) = match media {
        nim::ai::Media::Path { name, path, summary } => {
            let metadata = tokio::fs::metadata(&path).await?;
            let name = name
                .or_else(|| path.file_name().map(|value| value.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "response".into());
            (
                name,
                nim::state::ArtifactContent::File {
                    path,
                    size: metadata.len(),
                    media_type: Some(media_type.into()),
                },
                summary,
            )
        }
        nim::ai::Media::Url { name, url, summary } => {
            let name = name
                .or_else(|| {
                    url.path_segments()
                        .and_then(|mut values| values.next_back())
                        .map(String::from)
                })
                .unwrap_or_else(|| "response".into());
            (
                name,
                nim::state::ArtifactContent::Resource {
                    url: url.to_string(),
                    headers: Default::default(),
                },
                summary,
            )
        }
    };

    let artifact = ctx
        .services()
        .artifacts()
        .create(nim::state::ArtifactCreate {
            name,
            content,
            summary,
            embedding: None,
            confidence: None,
            salience: None,
            strength: None,
        })
        .await?;
    Ok(artifact.id)
}
