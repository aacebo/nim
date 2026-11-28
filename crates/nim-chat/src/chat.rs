use std::collections::BTreeMap;

use crate::{ChatMessage, Document, Tool};

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Chat {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<Tool>,
    pub documents: Vec<Document>,
    pub thinking: bool,
    pub vision: bool,
    pub audio: bool,
    pub max_tokens: Option<usize>,
    pub beams: Option<usize>,

    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Chat {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_tokens == Some(0) {
            return Err(nim_error::Diagnostic::message(
                "beams and max_tokens must be greater than zero",
            ));
        }

        Ok(())
    }
}
