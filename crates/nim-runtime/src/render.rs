use std::io::Write;

use nim_state::{Artifact, MessageContent, Services};
use nim_store::ArtifactContent;

use crate::Error;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Render {
    type Error: std::error::Error;

    fn render<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Self::Error>>;
}

impl Render for MessageContent {
    type Error = Error;

    fn render<'a>(&'a self, s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Error>> {
        Box::pin(async move {
            match self.clone() {
                Self::String(v) => Ok(v),
                Self::Text { text } => Ok(text),
                Self::Url { url } => Ok(url),
                Self::Base64 { base64 } => Ok(base64),
                Self::Artifact { artifact_id } => {
                    let artifact = s.artifacts().find_by_id(artifact_id).await?;
                    artifact.render(s).await
                }
                Self::Array(items) => {
                    let mut f = Vec::new();

                    for (i, item) in items.iter().enumerate() {
                        let v = item.render(s).await?;

                        if i < items.len() - 1 {
                            writeln!(&mut f, "{v}")?;
                        } else {
                            write!(&mut f, "{v}")?;
                        }
                    }

                    Ok(String::from_utf8(f)?)
                }
                _ => Err(Error::custom("render: unsupported message content")),
            }
        })
    }
}

impl Render for Artifact {
    type Error = Error;

    fn render<'a>(&'a self, _s: &'a Services<'_>) -> BoxFuture<'a, Result<String, Error>> {
        Box::pin(async move {
            match &self.content {
                ArtifactContent::Text { text } => Ok(text.clone()),
                ArtifactContent::File {
                    path,
                    size: _,
                    media_type,
                } => {
                    let mime_type = media_type.as_ref().map(|t| format!(r#" "{t}""#)).unwrap_or("".into());
                    Ok(format!("[{}](artifact://{}{})", self.name, path.display(), mime_type))
                }
            }
        })
    }
}
