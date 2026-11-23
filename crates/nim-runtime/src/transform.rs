use nim_state::{ArtifactContent, Role};
use nim_state::{Message, MessageContent, MessageContentPart, Services};

use crate::{BoxFuture, Transform};

impl Transform for Vec<Message> {
    type Output = Vec<nim_ai::ChatMessage>;
    type Context<'a> = Services<'a>;

    fn transform<'a>(&'a self, s: &'a Services<'a>) -> BoxFuture<'a, nim_error::Result<Self::Output>> {
        Box::pin(async move {
            let mut items = Vec::new();

            for item in self {
                items.push(item.transform(s).await?);
            }

            Ok(items)
        })
    }
}

impl Transform for Message {
    type Output = nim_ai::ChatMessage;
    type Context<'a> = Services<'a>;

    fn transform<'a>(&'a self, s: &'a Services<'a>) -> BoxFuture<'a, nim_error::Result<Self::Output>> {
        Box::pin(async move {
            match self.role {
                Role::System => Ok(nim_ai::ChatSystemMessage {
                    content: self.content.transform(s).await?,
                }
                .into()),
                Role::User => Ok(nim_ai::ChatUserMessage {
                    content: self.content.transform(s).await?,
                }
                .into()),
                Role::Agent => Ok(nim_ai::ChatAssistantMessage {
                    content: Some(self.content.transform(s).await?),
                    reasoning: None,
                    tool_calls: Default::default(),
                    annotations: Default::default(),
                }
                .into()),
                v => Err(nim_error::Diagnostic::message(format!("unsupported message role `{v}`"))),
            }
        })
    }
}

impl Transform for MessageContent {
    type Output = nim_ai::ChatMessageContent;
    type Context<'a> = Services<'a>;

    fn transform<'a>(&'a self, s: &'a Services<'a>) -> BoxFuture<'a, nim_error::Result<Self::Output>> {
        Box::pin(async move {
            match self {
                Self::String(v) => Ok(nim_ai::ChatMessageContent::Text(v.clone())),
                Self::Parts(parts) => {
                    let mut errors = Vec::new();
                    let mut out = Vec::new();

                    for part in parts {
                        match part.transform(s).await {
                            Err(err) => errors.push(err.source().clone()),
                            Ok(v) => out.push(v),
                        };
                    }

                    if !errors.is_empty() {
                        return Err(nim_error::Diagnostic::new().append(errors));
                    }

                    Ok(nim_ai::ChatMessageContent::Parts(out))
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content")),
            }
        })
    }
}

impl Transform for MessageContentPart {
    type Output = nim_ai::ChatMessageContentPart;
    type Context<'a> = Services<'a>;

    fn transform<'a>(&'a self, s: &'a Services<'a>) -> BoxFuture<'a, nim_error::Result<Self::Output>> {
        Box::pin(async move {
            match self {
                Self::Text { text } => Ok(nim_ai::ChatMessageContentPart::Text { text: text.clone() }),
                Self::Json { json } => Ok(nim_ai::ChatMessageContentPart::Json { json: json.clone() }),
                Self::Url { url } => Ok(nim_ai::ChatMessageContentPart::Url { url: url.clone() }),
                Self::Artifact { id, .. } => {
                    let artifact = s.artifacts().find_by_id(*id).await?;

                    match &artifact.content {
                        ArtifactContent::Text { text } => Ok(nim_ai::ChatMessageContentPart::Text { text: text.clone() }),
                        ArtifactContent::File { path, size, media_type } => Ok(match media_type {
                            Some(media_type) if media_type.starts_with("image") => {
                                nim_ai::ChatMessageContentPart::Image(nim_ai::Media::Path {
                                    name: Some(artifact.name),
                                    path: path.clone(),
                                    size: (*size).try_into().ok(),
                                    mime_type: Some(media_type.clone()),
                                    summary: artifact.summary,
                                })
                            }
                            Some(media_type) if media_type.starts_with("audio") => {
                                nim_ai::ChatMessageContentPart::Audio(nim_ai::Media::Path {
                                    name: Some(artifact.name),
                                    path: path.clone(),
                                    size: (*size).try_into().ok(),
                                    mime_type: Some(media_type.clone()),
                                    summary: artifact.summary,
                                })
                            }
                            Some(media_type) if media_type.starts_with("video") => {
                                nim_ai::ChatMessageContentPart::Video(nim_ai::Media::Path {
                                    name: Some(artifact.name),
                                    path: path.clone(),
                                    size: (*size).try_into().ok(),
                                    mime_type: Some(media_type.clone()),
                                    summary: artifact.summary,
                                })
                            }
                            _ => return Err(nim_error::Diagnostic::message("render: invalid artifact without media_type")),
                        }),
                        ArtifactContent::Resource {
                            url, size, media_type, ..
                        } => Ok(match url::Url::parse(url) {
                            Err(err) => return Err(nim_error::Diagnostic::error(err)),
                            Ok(url) => {
                                let kind = match media_type
                                    .as_deref()
                                    .and_then(|value| value.parse::<mime_guess::mime::Mime>().ok())
                                {
                                    Some(value) => value.type_().to_string(),
                                    None => {
                                        let Some(ext) = url
                                            .path_segments()
                                            .and_then(|mut segments| segments.next_back())
                                            .and_then(|filename| std::path::Path::new(filename).extension())
                                            .and_then(|ext| ext.to_str())
                                        else {
                                            return Err(nim_error::Diagnostic::message(
                                                "render: no artifact media_type or file extension for resource",
                                            ));
                                        };

                                        mime_guess::from_ext(ext).first_or_text_plain().type_().to_string()
                                    }
                                };

                                match kind.as_str() {
                                    "text" => nim_ai::ChatMessageContentPart::Text { text: url.to_string() },
                                    "image" => nim_ai::ChatMessageContentPart::Image(nim_ai::Media::Url {
                                        name: Some(artifact.name),
                                        url,
                                        size: size.and_then(|value| value.try_into().ok()),
                                        mime_type: media_type.clone(),
                                        summary: artifact.summary,
                                    }),
                                    "audio" => nim_ai::ChatMessageContentPart::Audio(nim_ai::Media::Url {
                                        name: Some(artifact.name),
                                        url,
                                        size: size.and_then(|value| value.try_into().ok()),
                                        mime_type: media_type.clone(),
                                        summary: artifact.summary,
                                    }),
                                    "video" => nim_ai::ChatMessageContentPart::Video(nim_ai::Media::Url {
                                        name: Some(artifact.name),
                                        url,
                                        size: size.and_then(|value| value.try_into().ok()),
                                        mime_type: media_type.clone(),
                                        summary: artifact.summary,
                                    }),
                                    v => return Err(nim_error::Diagnostic::message(format!("unsupported mime_type {v}"))),
                                }
                            }
                        }),
                    }
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content part")),
            }
        })
    }
}
