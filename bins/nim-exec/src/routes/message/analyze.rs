use nim::{runtime::Transform, signals::Signal, state::Message};

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
            reply_to_id: message.reply_to_id,
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
                    nim::ai::ChatMessageContentPart::Json { json } => {
                        output.push(nim::state::MessageContentPart::Json { json });
                    }
                    nim::ai::ChatMessageContentPart::Url { url } => {
                        output.push(nim::state::MessageContentPart::Url { url });
                    }
                    nim::ai::ChatMessageContentPart::Image(media) => {
                        output.push(media_artifact(persist_media(ctx, media, "image/png").await?));
                    }
                    nim::ai::ChatMessageContentPart::Audio(media) => {
                        output.push(media_artifact(persist_media(ctx, media, "audio/mpeg").await?));
                    }
                    nim::ai::ChatMessageContentPart::Video(media) => {
                        output.push(media_artifact(persist_media(ctx, media, "video/mp4").await?));
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

fn media_artifact(artifact: nim::state::Artifact) -> nim::state::MessageContentPart {
    let (size, media_type) = match artifact.content {
        nim::state::ArtifactContent::File { size, media_type, .. } => (Some(size), media_type),
        nim::state::ArtifactContent::Resource { size, media_type, .. } => (size, media_type),
        nim::state::ArtifactContent::Text { .. } => (None, None),
    };

    nim::state::MessageContentPart::Artifact {
        id: artifact.id,
        name: Some(artifact.name),
        size,
        media_type,
    }
}

async fn persist_media(ctx: &SignalContext, media: nim::ai::Media, media_type: &str) -> nim::error::Result<nim::state::Artifact> {
    let (name, content, summary) = match media {
        nim::ai::Media::Path {
            name,
            path,
            mime_type,
            summary,
            ..
        } => {
            let metadata = tokio::fs::metadata(&path).await?;
            let name = name
                .or_else(|| path.file_name().map(|value| value.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "response".into());
            (
                name,
                nim::state::ArtifactContent::File {
                    path,
                    size: metadata.len(),
                    media_type: mime_type.or_else(|| Some(media_type.into())),
                },
                summary,
            )
        }
        nim::ai::Media::Url {
            name,
            url,
            size,
            mime_type,
            summary,
        } => {
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
                    size: size.and_then(|value| value.try_into().ok()),
                    media_type: mime_type.or_else(|| Some(media_type.into())),
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
    Ok(artifact)
}
