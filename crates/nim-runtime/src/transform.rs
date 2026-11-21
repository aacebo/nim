use nim_state::{Message, MessageContent, MessageContentPart, Services};
use nim_store::{ArtifactContent, Role};

use crate::BoxFuture;

pub trait Transform {
    type Out;

    fn transform<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, nim_error::Result<Self::Out>>;
}

impl Transform for Vec<Message> {
    type Out = Vec<nim_ai::ChatMessage>;

    fn transform<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, nim_error::Result<Self::Out>> {
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
    type Out = nim_ai::ChatMessage;

    fn transform<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, nim_error::Result<Self::Out>> {
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
    type Out = nim_ai::ChatMessageContent;

    fn transform<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, nim_error::Result<Self::Out>> {
        Box::pin(async move {
            match self {
                Self::Single(item) => Ok(nim_ai::ChatMessageContent::Parts(vec![item.transform(s).await?])),
                Self::Array(items) => {
                    let mut errors = Vec::new();
                    let mut out = Vec::new();

                    for item in items {
                        match item.transform(s).await {
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
    type Out = nim_ai::ChatMessageContentPart;

    fn transform<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, nim_error::Result<Self::Out>> {
        Box::pin(async move {
            match self {
                Self::String(text) | Self::Text { text } => Ok(nim_ai::ChatMessageContentPart::Text { text: text.clone() }),
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
                                    path: path.clone(),
                                    summary: artifact.summary,
                                })
                            }
                            Some(media_type) if media_type.starts_with("audio") => {
                                nim_ai::ChatMessageContentPart::Audio(nim_ai::Media::Path {
                                    path: path.clone(),
                                    summary: artifact.summary,
                                })
                            }
                            Some(media_type) if media_type.starts_with("video") => {
                                nim_ai::ChatMessageContentPart::Video(nim_ai::Media::Path {
                                    path: path.clone(),
                                    summary: artifact.summary,
                                })
                            }
                            _ => return Err(nim_error::Diagnostic::message("render: invalid artifact without media_type")),
                        }),
                    }
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content part")),
            }
        })
    }
}
