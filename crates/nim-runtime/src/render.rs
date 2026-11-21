use nim_state::{Artifact, Message, MessageContent, MessageContentPart, Services};
use nim_store::ArtifactContent;

use crate::BoxFuture;

pub trait Render {
    type Error;

    fn render<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Self::Error>>;
}

impl Render for Message {
    type Error = nim_error::Diagnostic;

    fn render<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move { self.content.render(s).await })
    }
}

impl Render for MessageContent {
    type Error = nim_error::Diagnostic;

    fn render<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move {
            match self {
                Self::Single(v) => v.render(s).await,
                Self::Array(items) => {
                    let mut lines = Vec::new();

                    for item in items {
                        lines.push(item.render(s).await?);
                    }

                    Ok(lines.join("\n"))
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content")),
            }
        })
    }
}

impl Render for MessageContentPart {
    type Error = nim_error::Diagnostic;

    fn render<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move {
            match self {
                Self::String(v) => Ok(v.clone()),
                Self::Text { text } => Ok(text.clone()),
                Self::Artifact { artifact_id } => {
                    let artifact = s.artifacts().find_by_id(*artifact_id).await?;
                    artifact.render(s).await
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content")),
            }
        })
    }
}

impl Render for Artifact {
    type Error = nim_error::Diagnostic;

    fn render<'a>(&'a self, _s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Self::Error>> {
        Box::pin(async move {
            match &self.summary {
                Some(summary) => Ok(summary.clone()),
                None => match &self.content {
                    ArtifactContent::Text { text } => Ok(text.clone()),
                    ArtifactContent::File {
                        path,
                        size: _,
                        media_type,
                    } => {
                        let mime_type = media_type.as_ref().map(|t| format!(r#" "{t}""#)).unwrap_or("".into());
                        Ok(format!("[{}](artifact://{}{})", self.name, path.display(), mime_type))
                    }
                },
            }
        })
    }
}
