use crate::{Media, widgets};

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
