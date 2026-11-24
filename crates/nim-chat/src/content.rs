use crate::Media;

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
    Text {
        text: String,
    },
    Json {
        json: serde_json::Value,
    },
    Url {
        url: url::Url,
    },
    Image {
        #[serde(flatten)]
        media: Media,
    },
    Audio {
        #[serde(flatten)]
        media: Media,
    },
    Video {
        #[serde(flatten)]
        media: Media,
    },
    Artifact {
        id: String,
        name: String,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<u64>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
}
