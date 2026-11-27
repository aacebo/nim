use crate::{Media, Parse, ParseMode, Parser, Tag, widgets};

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
            ChatMessageContent::String(_) => 1,
            ChatMessageContent::Parts(parts) => parts.len(),
        }
    }

    pub fn push(&mut self, value: impl Into<ChatStringOrContentPart>) {
        match value.into() {
            ChatStringOrContentPart::String(text) | ChatStringOrContentPart::Part(ChatMessageContentPart::Text { text }) => {
                match self {
                    ChatMessageContent::String(curr) => curr.push_str(&text),
                    ChatMessageContent::Parts(parts) => match parts.last_mut() {
                        None => *self = ChatMessageContent::String(text),
                        Some(ChatMessageContentPart::Text { text: curr }) => curr.push_str(&text),
                        Some(_) => parts.push(ChatMessageContentPart::Text { text }),
                    },
                }
            }
            ChatStringOrContentPart::Part(part) => match self {
                ChatMessageContent::String(curr) => {
                    let text = std::mem::take(curr);
                    *self = ChatMessageContent::Parts(vec![ChatMessageContentPart::Text { text }, part]);
                }
                ChatMessageContent::Parts(parts) => parts.push(part),
            },
        }
    }
}

impl std::str::FromStr for ChatMessageContent {
    type Err = nim_error::Diagnostic;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let mut p = Parser::from_src(src, ParseMode::Lenient);
        ChatMessageContent::parse(&mut p)
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
        let src = p.remaining();
        let start = p.span().start();

        if p.starts_with(Tag::Think.open()) {
            let mut parser = p.clone();
            parser.parse_open_tag(Tag::Think)?;

            if let Some(thinking) = parser.take_until_exact(Tag::Think.close()) {
                let thinking = thinking.trim().to_string();
                parser.parse_close_tag(Tag::Think)?;
                *p = parser;
                return Ok(ChatMessageContentPart::Thinking { thinking });
            }

            if *p.mode() == ParseMode::Strict {
                return Err(nim_error::Diagnostic::message("unterminated reasoning block"));
            }
        }

        if let Ok(p) = p.parse_open_tag(Tag::Widget) {
            p.take_while(char::is_whitespace);

            let serde_json::Value::Object(object) = p.parse_json()? else {
                return Err(p.error("widget json payload required after opening widget tag"));
            };

            let Some(serde_json::Value::String(ty)) = object.get("type") else {
                return Err(nim_error::Diagnostic::message("invalid widget json payload"));
            };

            if ty != "widget" {
                return Err(nim_error::Diagnostic::message("invalid widget json payload"));
            }

            p.take_while(char::is_whitespace);
            p.parse_close_tag(Tag::Widget)?;
            let widget: widgets::Widget = serde_json::from_value(serde_json::Value::Object(object))?;
            return Ok(ChatMessageContentPart::Widget(widget));
        }

        if let Some(url) = p.parse_url() {
            return Ok(ChatMessageContentPart::Url { url });
        }

        if p.peek_json().is_some() {
            let json = p.parse_json()?;
            return Ok(ChatMessageContentPart::Json { json });
        }

        let anchor = Parser::from_src(src, *p.mode());
        let mut scanner = anchor.clone();

        while !scanner.is_eof() {
            let index = scanner.span().start();

            let previous = scanner.prev();
            let boundary = previous.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_');

            if index > 0
                && (scanner.starts_with(Tag::Think.open())
                    || scanner.starts_with(Tag::Widget.open())
                    || boundary && scanner.peek_url().is_some()
                    || scanner.peek_json().is_some())
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
        Ok(ChatMessageContentPart::Text { text })
    }
}
