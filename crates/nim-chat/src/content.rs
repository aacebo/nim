use crate::{Media, widgets};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ChatMessageContent {
    String(String),
    Parts(Vec<ChatMessageContentPart>),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatMessageContentPart {
    /// plain text
    Text { text: String },

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
        name: String,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<u64>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
}
