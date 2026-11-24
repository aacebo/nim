use std::collections::BTreeMap;

use crate::{
    ChatMessageContent,
    widgets::{Element, Surface},
};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// reset host element to default state
    Reset,

    /// submit action with host element value
    Submit { action_id: String, surface: Surface },

    /// open url in new browser tab
    OpenUrl { url: url::Url },

    /// open a dialog
    OpenDialog {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        header: Option<Vec<Element>>,
        body: Vec<Element>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        footer: Option<Vec<Element>>,
    },

    /// log to console
    Log { text: String },

    /// send a message
    Send {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reply_to_id: Option<String>,
        content: ChatMessageContent,
    },

    /// copy text to clipboard
    Copy { text: String },

    /// send an http request
    Http {
        url: url::Url,
        method: HttpMethod,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        headers: Option<BTreeMap<String, String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body: Option<serde_json::Value>,
    },
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}
