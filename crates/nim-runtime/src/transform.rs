use nim_state::{Message, MessageContent, MessageContentPart, Services};
use nim_store::{ArtifactContent, Role};

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
                Self::Artifact { artifact_id } => {
                    let artifact = s.artifacts().find_by_id(*artifact_id).await?;

                    match &artifact.content {
                        ArtifactContent::Text { text } => Ok(nim_ai::ChatMessageContentPart::Text { text: text.clone() }),
                        ArtifactContent::File {
                            path,
                            size: _,
                            media_type,
                        } => Ok(match media_type {
                            Some(media_type) if media_type.starts_with("image") => {
                                nim_ai::ChatMessageContentPart::Image(nim_ai::Media::Path {
                                    name: Some(artifact.name),
                                    path: path.clone(),
                                    summary: artifact.summary,
                                })
                            }
                            Some(media_type) if media_type.starts_with("audio") => {
                                nim_ai::ChatMessageContentPart::Audio(nim_ai::Media::Path {
                                    name: Some(artifact.name),
                                    path: path.clone(),
                                    summary: artifact.summary,
                                })
                            }
                            Some(media_type) if media_type.starts_with("video") => {
                                nim_ai::ChatMessageContentPart::Video(nim_ai::Media::Path {
                                    name: Some(artifact.name),
                                    path: path.clone(),
                                    summary: artifact.summary,
                                })
                            }
                            _ => return Err(nim_error::Diagnostic::message("render: invalid artifact without media_type")),
                        }),
                        ArtifactContent::Resource { url, headers: _ } => Ok(match url::Url::parse(url) {
                            Err(err) => return Err(nim_error::Diagnostic::error(err)),
                            Ok(url) => {
                                let Some(ext) = url
                                    .path_segments()
                                    .and_then(|mut segments| segments.next_back())
                                    .and_then(|filename| std::path::Path::new(filename).extension())
                                    .and_then(|ext| ext.to_str())
                                else {
                                    return Err(nim_error::Diagnostic::message(
                                        "render: no artifact file extension for resource",
                                    ));
                                };

                                let guess = mime_guess::from_ext(ext);

                                match guess.first_or_text_plain().type_() {
                                    mime_guess::mime::TEXT => nim_ai::ChatMessageContentPart::Text { text: url.to_string() },
                                    mime_guess::mime::IMAGE => nim_ai::ChatMessageContentPart::Image(nim_ai::Media::Url {
                                        name: Some(artifact.name),
                                        url,
                                        summary: artifact.summary,
                                    }),
                                    mime_guess::mime::AUDIO => nim_ai::ChatMessageContentPart::Audio(nim_ai::Media::Url {
                                        name: Some(artifact.name),
                                        url,
                                        summary: artifact.summary,
                                    }),
                                    mime_guess::mime::VIDEO => nim_ai::ChatMessageContentPart::Video(nim_ai::Media::Url {
                                        name: Some(artifact.name),
                                        url,
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
