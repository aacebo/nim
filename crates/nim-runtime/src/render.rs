use nim_state::ArtifactContent;
use nim_state::{Artifact, Message, MessageContent, MessageContentPart, Services};

use crate::{BoxFuture, Render};

impl Render for Message {
    type Context<'a> = Services<'a>;

    fn render<'a>(&'a self, s: &'a Self::Context<'a>) -> BoxFuture<'a, Result<String, nim_error::Diagnostic>> {
        Box::pin(async move { self.content.render(s).await })
    }
}

impl Render for MessageContent {
    type Context<'a> = Services<'a>;

    fn render<'a>(&'a self, s: &'a Services<'a>) -> BoxFuture<'a, Result<String, nim_error::Diagnostic>> {
        Box::pin(async move {
            match self {
                Self::String(v) => Ok(v.clone()),
                Self::Parts(parts) => {
                    let mut lines = Vec::new();

                    for part in parts {
                        lines.push(part.render(s).await?);
                    }

                    Ok(lines.join("\n"))
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content")),
            }
        })
    }
}

impl Render for MessageContentPart {
    type Context<'a> = Services<'a>;

    fn render<'a>(&'a self, s: &'a Services<'a>) -> BoxFuture<'a, Result<String, nim_error::Diagnostic>> {
        Box::pin(async move {
            match self {
                Self::Text { text } => Ok(text.clone()),
                Self::Url { url } => Ok(url.to_string()),
                Self::Json { json } => Ok(json.to_string()),
                Self::Artifact { id, .. } => {
                    let artifact = s.artifacts().find_by_id(*id).await?;
                    artifact.render(s).await
                }
                _ => Err(nim_error::Diagnostic::message("render: unsupported message content")),
            }
        })
    }
}

impl Render for Artifact {
    type Context<'a> = Services<'a>;

    fn render<'a>(&'a self, _s: &'a Services<'a>) -> BoxFuture<'a, Result<String, nim_error::Diagnostic>> {
        Box::pin(async move {
            match &self.summary {
                Some(summary) => Ok(summary.clone()),
                None => match &self.content {
                    ArtifactContent::Text { text } => Ok(text.clone()),
                    ArtifactContent::Resource { url, .. } => Ok(format!("[{}]({})", self.name, url,)),
                    ArtifactContent::File {
                        path,
                        size: _,
                        media_type,
                    } => {
                        let mime_type = media_type.as_ref().map(|t| format!(r#" "{t}""#)).unwrap_or("".into());
                        Ok(format!("[{}](file://{}{})", self.name, path.display(), mime_type))
                    }
                },
            }
        })
    }
}
