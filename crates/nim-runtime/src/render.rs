use nim_chat::{
    ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatSystemMessage, ChatUserMessage, Media,
};
use nim_state::{ArtifactContent, ArtifactId, Message, Role, Services};

pub async fn render_message(services: &Services<'_>, message: &Message) -> nim_error::Result<ChatMessage> {
    let content = match &message.content {
        ChatMessageContent::String(text) => ChatMessageContent::String(text.clone()),
        ChatMessageContent::Parts(parts) => {
            let mut output = Vec::with_capacity(parts.len());
            let mut errors = Vec::new();

            for part in parts {
                match render_part(services, part).await {
                    Ok(value) => output.push(value),
                    Err(error) => errors.push(error.source().clone()),
                }
            }

            if !errors.is_empty() {
                return Err(nim_error::Diagnostic::new().append(errors));
            }

            ChatMessageContent::Parts(output)
        }
        _ => return Err(nim_error::Diagnostic::message("unsupported message content")),
    };

    match message.role {
        Role::System => Ok(ChatSystemMessage {
            name: None,
            content,
            extra: Default::default(),
        }
        .into()),
        Role::User => Ok(ChatUserMessage {
            name: None,
            content,
            extra: Default::default(),
        }
        .into()),
        Role::Agent => Ok(ChatAgentMessage {
            name: None,
            content: Some(content),
            annotations: Default::default(),
            tool_calls: Default::default(),
            extra: Default::default(),
        }
        .into()),
        _ => Err(nim_error::Diagnostic::message("unsupported message role")),
    }
}

pub async fn render_message_text(services: &Services<'_>, message: &Message) -> nim_error::Result<String> {
    match &message.content {
        ChatMessageContent::String(text) => Ok(text.clone()),
        ChatMessageContent::Parts(parts) => {
            let mut output = Vec::with_capacity(parts.len());

            for part in parts {
                if matches!(part, ChatMessageContentPart::Thinking { .. }) {
                    continue;
                }

                output.push(match part {
                    ChatMessageContentPart::Text { text } => text.clone(),
                    ChatMessageContentPart::Json { json } => json.to_string(),
                    ChatMessageContentPart::Url { url } => url.to_string(),
                    ChatMessageContentPart::Speak { speak } => speak.clone(),
                    ChatMessageContentPart::Image { media }
                    | ChatMessageContentPart::Audio { media }
                    | ChatMessageContentPart::Video { media } => render_media_text(media)?,
                    ChatMessageContentPart::Artifact { id, .. } => {
                        let id: ArtifactId = id.parse()?;
                        let artifact = services.artifacts().find_by_id(id).await?;

                        match artifact.summary {
                            Some(summary) => summary,
                            None => match artifact.content {
                                ArtifactContent::Text { text } => text,
                                ArtifactContent::Resource { url, .. } => format!("[{}]({})", artifact.name, url),
                                ArtifactContent::File { path, media_type, .. } => {
                                    let media_type =
                                        media_type.as_ref().map(|value| format!(r#" "{value}""#)).unwrap_or_default();
                                    format!("[{}](file://{}{})", artifact.name, path.display(), media_type)
                                }
                            },
                        }
                    }
                    _ => return Err(nim_error::Diagnostic::message("unsupported message content part")),
                });
            }

            Ok(output.join(""))
        }
        _ => Err(nim_error::Diagnostic::message("unsupported message content")),
    }
}

async fn render_part(services: &Services<'_>, part: &ChatMessageContentPart) -> nim_error::Result<ChatMessageContentPart> {
    let ChatMessageContentPart::Artifact { id, .. } = part else {
        return Ok(part.clone());
    };

    let id: ArtifactId = id.parse()?;
    let artifact = services.artifacts().find_by_id(id).await?;
    let (media, media_type) = match artifact.content {
        ArtifactContent::Text { text } => return Ok(ChatMessageContentPart::Text { text }),
        ArtifactContent::File { path, size, media_type } => (
            Media::Path {
                path,
                name: Some(artifact.name),
                size: size.try_into().ok(),
                mime_type: media_type.clone(),
                summary: artifact.summary,
            },
            media_type,
        ),
        ArtifactContent::Resource {
            url, size, media_type, ..
        } => {
            let url: url::Url = url.parse().map_err(nim_error::Diagnostic::error)?;

            if media_type.as_deref().is_some_and(|value| value.starts_with("text/")) {
                return Ok(ChatMessageContentPart::Text { text: url.to_string() });
            }

            (
                Media::Url {
                    url,
                    name: Some(artifact.name),
                    size: size.and_then(|value| value.try_into().ok()),
                    mime_type: media_type.clone(),
                    summary: artifact.summary,
                },
                media_type,
            )
        }
    };

    match media_type.as_deref().and_then(|value| value.split('/').next()) {
        Some("image") => Ok(ChatMessageContentPart::Image { media }),
        Some("audio") => Ok(ChatMessageContentPart::Audio { media }),
        Some("video") => Ok(ChatMessageContentPart::Video { media }),
        _ => Err(nim_error::Diagnostic::message("render: invalid artifact without media_type")),
    }
}

fn render_media_text(media: &Media) -> nim_error::Result<String> {
    match media {
        Media::Path {
            name,
            path,
            mime_type,
            summary,
            ..
        } => match summary {
            Some(summary) => Ok(summary.clone()),
            None => {
                let name = name
                    .as_deref()
                    .or_else(|| path.file_name().and_then(|value| value.to_str()))
                    .unwrap_or("response");
                let media_type = mime_type.as_ref().map(|value| format!(r#" "{value}""#)).unwrap_or_default();
                Ok(format!("[{name}](file://{}{media_type})", path.display()))
            }
        },
        Media::Url { name, url, summary, .. } => match summary {
            Some(summary) => Ok(summary.clone()),
            None => Ok(format!("[{}]({url})", name.as_deref().unwrap_or("response"))),
        },
        Media::Data { name, summary, .. } => Ok(summary.clone().or_else(|| name.clone()).unwrap_or_else(|| "media".into())),
        _ => Err(nim_error::Diagnostic::message("unsupported media")),
    }
}
