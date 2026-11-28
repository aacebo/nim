use crate::{Media, Parse, Parser, Tag, render::Render, widgets};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
#[serde(untagged)]
pub enum ChatMessageContent {
    #[validate]
    String(#[validate(min_length = 1)] String),

    #[validate(min_items = 1)]
    Parts(#[validate] Vec<ChatMessageContentPart>),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatMessageContentPart {
    /// represents a self reflection or reasoning that
    /// is not typically rendered as message content
    Thinking { thinking: String },

    /// plain text
    Text {
        #[validate(min_length = 1)]
        text: String,
    },

    /// structured JSON object or array
    Json { json: serde_json::Value },

    /// ex. https://google.com
    Url { url: url::Url },

    /// image data/reference
    Image {
        #[serde(flatten)]
        media: Media,
    },

    /// audio data/reference
    Audio {
        #[serde(flatten)]
        media: Media,
    },

    /// video data/reference
    Video {
        #[serde(flatten)]
        media: Media,
    },

    /// say something using text -> speech
    Speak { speak: String },

    /// rich ui elements that get rendered for user interaction
    Widget(widgets::Widget),

    /// a reference to some shared artifact
    Artifact {
        id: String,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<u64>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
}

#[non_exhaustive]
pub enum ChatStringOrContentPart {
    String(String),
    Part(ChatMessageContentPart),
}

impl<T: std::fmt::Display> From<T> for ChatStringOrContentPart {
    fn from(value: T) -> Self {
        Self::String(value.to_string())
    }
}

impl From<ChatMessageContentPart> for ChatStringOrContentPart {
    fn from(value: ChatMessageContentPart) -> Self {
        Self::Part(value)
    }
}

impl ChatMessageContent {
    pub fn count(&self) -> usize {
        match self {
            Self::String(_) => 1,
            Self::Parts(parts) => parts.len(),
        }
    }

    pub fn push(&mut self, value: impl Into<ChatStringOrContentPart>) {
        match value.into() {
            ChatStringOrContentPart::String(text) | ChatStringOrContentPart::Part(ChatMessageContentPart::Text { text }) => {
                match self {
                    Self::String(curr) => curr.push_str(&text),
                    Self::Parts(parts) => match parts.last_mut() {
                        None => *self = Self::String(text),
                        Some(ChatMessageContentPart::Text { text: curr }) => curr.push_str(&text),
                        Some(_) => parts.push(ChatMessageContentPart::Text { text }),
                    },
                }
            }
            ChatStringOrContentPart::Part(part) => match self {
                Self::String(curr) => {
                    let text = std::mem::take(curr);
                    *self = Self::Parts(vec![ChatMessageContentPart::Text { text }, part]);
                }
                Self::Parts(parts) => parts.push(part),
            },
        }
    }
}

impl std::str::FromStr for ChatMessageContent {
    type Err = nim_error::Diagnostic;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Self::parse(&mut Parser::from_src(src))
    }
}

impl Parse for ChatMessageContent {
    type Error = nim_error::Diagnostic;

    fn parse(p: &mut Parser) -> Result<Self, Self::Error> {
        let mut content = Self::Parts(Vec::new());

        while !p.is_eof() {
            content.push(ChatMessageContentPart::parse(p)?);
        }

        if content.count() == 0 {
            return Err(nim_error::Diagnostic::message("message content is empty"));
        }

        Ok(content)
    }
}

impl Parse for ChatMessageContentPart {
    type Error = nim_error::Diagnostic;

    fn parse(p: &mut Parser) -> Result<Self, Self::Error> {
        if p.starts_with_tag() {
            let mut parser = p.clone();
            let part = (|| {
                let tag = [
                    Tag::Think,
                    Tag::Json,
                    Tag::Widget,
                    Tag::Image,
                    Tag::Audio,
                    Tag::Video,
                    Tag::Speak,
                    Tag::Artifact,
                ]
                .into_iter()
                .find(|tag| parser.starts_with(tag.open()))?;
                parser.parse_open_tag(tag).ok()?;
                let raw = parser.take_until_exact(tag.close())?;
                parser.parse_close_tag(tag).ok()?;

                match tag {
                    Tag::Think => Some(Self::Thinking {
                        thinking: raw.to_string(),
                    }),
                    Tag::Json => Some(Self::Json {
                        json: serde_json::from_str(raw.trim()).ok()?,
                    }),
                    Tag::Widget => Some(Self::Widget(serde_json::from_str(raw.trim()).ok()?)),
                    Tag::Image => Some(Self::Image {
                        media: serde_json::from_str(raw.trim()).ok()?,
                    }),
                    Tag::Audio => Some(Self::Audio {
                        media: serde_json::from_str(raw.trim()).ok()?,
                    }),
                    Tag::Video => Some(Self::Video {
                        media: serde_json::from_str(raw.trim()).ok()?,
                    }),
                    Tag::Speak => Some(Self::Speak {
                        speak: serde_json::from_str(raw.trim()).ok()?,
                    }),
                    Tag::Artifact => {
                        let artifact = serde_json::from_str::<serde_json::Value>(raw.trim()).ok()?;
                        let artifact = artifact.as_object()?;

                        Some(Self::Artifact {
                            id: artifact.get("id")?.as_str()?.to_string(),
                            name: artifact.get("name").and_then(serde_json::Value::as_str).map(str::to_string),
                            size: artifact.get("size").and_then(serde_json::Value::as_u64),
                            media_type: artifact
                                .get("media_type")
                                .and_then(serde_json::Value::as_str)
                                .map(str::to_string),
                        })
                    }
                    _ => None,
                }
            })();

            if let Some(part) = part {
                *p = parser;
                return Ok(part);
            }
        }

        if let Some(url) = p.parse_url() {
            return Ok(Self::Url { url });
        }

        if p.peek_json().is_some() {
            return Ok(Self::Json { json: p.parse_json()? });
        }

        let src = p.remaining();
        let start = p.span().start();
        let anchor = Parser::from_src(src);
        let mut scanner = anchor.clone();

        while !scanner.is_eof() {
            let index = scanner.span().start();
            let previous = scanner.prev();
            let boundary = previous.is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_');

            if index > 0
                && (scanner.starts_with_tag() || boundary && scanner.peek_url().is_some() || scanner.peek_json().is_some())
            {
                break;
            }

            scanner.take_char();
        }

        let index = scanner.span().start();

        if index == 0 {
            return Err(nim_error::Diagnostic::message("message content is empty"));
        }

        let text = p
            .take_to_offset(start + index)
            .ok_or_else(|| p.error("invalid message content"))?
            .to_string();
        Ok(Self::Text { text })
    }
}

impl Render for ChatMessageContent {
    type Error = nim_error::Diagnostic;

    fn render(&self, f: &mut crate::render::Formatter) -> Result<(), Self::Error> {
        match self {
            Self::String(text) => f.write(text).map_err(nim_error::IntoDiagnostic::into_diagnostic),
            Self::Parts(parts) => {
                for part in parts {
                    part.render(f)?;
                }

                Ok(())
            }
        }
    }
}

impl Render for ChatMessageContentPart {
    type Error = nim_error::Diagnostic;

    fn render(&self, f: &mut crate::render::Formatter) -> Result<(), Self::Error> {
        match self {
            Self::Thinking { thinking } => render_tag(f, Tag::Think, thinking),
            Self::Text { text } => f.write(text).map_err(nim_error::IntoDiagnostic::into_diagnostic),
            Self::Json { json } => render_json_tag(f, Tag::Json, json),
            Self::Url { url } => f.write(url).map_err(nim_error::IntoDiagnostic::into_diagnostic),
            Self::Widget(widget) => render_json_tag(f, Tag::Widget, widget),
            Self::Image { media } => render_json_tag(f, Tag::Image, media),
            Self::Audio { media } => render_json_tag(f, Tag::Audio, media),
            Self::Video { media } => render_json_tag(f, Tag::Video, media),
            Self::Speak { speak } => render_json_tag(f, Tag::Speak, speak),
            Self::Artifact {
                id,
                name,
                size,
                media_type,
            } => render_json_tag(
                f,
                Tag::Artifact,
                &serde_json::json!({
                    "id": id,
                    "name": name,
                    "size": size,
                    "media_type": media_type,
                }),
            ),
        }
    }
}

fn render_tag(f: &mut crate::render::Formatter, tag: Tag, value: impl std::fmt::Display) -> nim_error::Result<()> {
    f.write(tag.open()).map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.write(value).map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.write(tag.close()).map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

fn render_json_tag<T: serde::Serialize>(f: &mut crate::render::Formatter, tag: Tag, value: &T) -> nim_error::Result<()> {
    render_tag(f, tag, serde_json::to_string(value)?)
}
