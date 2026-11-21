use crate::tasks;
use crate::{
    ChatAssistantMessage, ChatOptions,
    models::{ModelRef, load},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,

    #[serde(flatten)]
    pub chat: ChatOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::chat(),
            chat: Default::default(),
        }
    }
}

#[tracing::instrument(skip_all, err)]
pub fn run(options: Options) -> nim_error::Result<ChatAssistantMessage> {
    let model = load(&options.model)?;
    let capable = model.as_chat().ok_or_else(|| model.cannot("chat"))?;
    tasks::chat(capable, &model.context(), options.chat)
}
